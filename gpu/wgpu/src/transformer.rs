//! Interprete GPU para la cinta de operaciones del Transformer no_std.
use boddy_transformer::Graph;
use std::time::Duration;
use wgpu::util::DeviceExt;
pub struct Trainer {
    graph: Graph,
    device: wgpu::Device,
    queue: wgpu::Queue,
    buffers: Vec<wgpu::Buffer>,
    bind: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    pub device_name: String,
    pub tensor_bytes: u64,
}
impl Trainer {
    pub fn new(graph: Graph, weights: &[f32]) -> Result<Self, String> {
        pollster::block_on(Self::create(graph, weights))
    }
    async fn create(graph: Graph, weights: &[f32]) -> Result<Self, String> {
        if weights.len() != graph.config().parameters() || weights.iter().any(|v| !v.is_finite()) {
            return Err("Pesos Transformer invalidos".into());
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        if info.device_type == wgpu::DeviceType::Cpu {
            return Err("Se requiere GPU real para entrenar".into());
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Boddy attention reference"),
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        let n = graph.instructions().len();
        let sizes = [
            32,
            n * 16,
            weights.len() * 4,
            graph.config().context * 4,
            n * 4,
            n * 4,
        ];
        let tensor_bytes = sizes.iter().sum::<usize>() as u64;
        let buffers: Vec<_> = sizes
            .iter()
            .enumerate()
            .map(|(i, size)| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Attention tensor"),
                    contents: &vec![0u8; *size],
                    usage: if i == 0 {
                        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
                    } else {
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC
                    },
                })
            })
            .collect();
        queue.write_buffer(&buffers[1], 0, bytemuck::cast_slice(graph.instructions()));
        queue.write_buffer(&buffers[2], 0, bytemuck::cast_slice(weights));
        let entries: Vec<_> = (0..6)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 0 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: matches!(binding, 1 | 3),
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &entries,
        });
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Attention autodiff tape"),
            source: wgpu::ShaderSource::Wgsl(include_str!("transformer.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &shader,
            entry_point: Some("execute"),
            compilation_options: Default::default(),
            cache: None,
        });
        if let Some(e) = scope.pop().await {
            return Err(e.to_string());
        }
        Ok(Self {
            graph,
            device,
            queue,
            buffers,
            bind,
            pipeline,
            device_name: info.name,
            tensor_bytes,
        })
    }
    pub fn step(&self, input: &[u32], target: u32, rate: f32) -> Result<(), String> {
        let c = self.graph.config();
        if input.len() != c.context
            || input.iter().any(|v| *v >= 256)
            || target >= 256
            || !rate.is_finite()
            || !(0.0..=0.1).contains(&rate)
        {
            return Err("Lote Transformer invalido".into());
        }
        let config = [
            self.graph.instructions().len() as u32,
            c.parameters() as u32,
            self.graph.loss_index() as u32,
            target,
            c.context as u32,
            c.dimension as u32,
            rate.to_bits(),
            0,
        ];
        self.queue
            .write_buffer(&self.buffers[0], 0, bytemuck::cast_slice(&config));
        self.queue
            .write_buffer(&self.buffers[3], 0, bytemuck::cast_slice(input));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        self.queue.submit([encoder.finish()]);
        Ok(())
    }
    fn read(&self, index: usize, count: usize, offset: u64) -> Result<Vec<f32>, String> {
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (count * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&self.buffers[index], offset, &output, 0, (count * 4) as u64);
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        output.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let bytes = output
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let values = bytemuck::cast_slice::<u8, f32>(&bytes).to_vec();
        drop(bytes);
        output.unmap();
        if values.iter().any(|v| !v.is_finite()) {
            return Err("Transformer GPU no finito".into());
        }
        Ok(values)
    }
    pub fn weights(&self) -> Result<Vec<f32>, String> {
        self.read(2, self.graph.config().parameters(), 0)
    }
    pub fn probe(&self, input: &[u32], target: u32) -> Result<(f32, Vec<f32>), String> {
        self.step(input, target, 0.0)?;
        let loss = self.read(4, 1, (self.graph.loss_index() * 4) as u64)?[0];
        Ok((loss, self.read(5, self.graph.config().parameters(), 0)?))
    }
}
