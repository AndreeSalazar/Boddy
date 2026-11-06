//! El adaptador WGPU no filtra Device/Queue/Buffer a las demás crates.
use boddy_neural::Config;
use std::time::Duration;
use wgpu::util::DeviceExt;
pub mod transformer;

pub struct Trainer {
    config: Config,
    batch: usize,
    device: wgpu::Device,
    queue: wgpu::Queue,
    buffers: Vec<wgpu::Buffer>,
    bind: wgpu::BindGroup,
    forward: wgpu::ComputePipeline,
    gradient: wgpu::ComputePipeline,
    update: wgpu::ComputePipeline,
    pub device_name: String,
    pub tensor_bytes: u64,
}

impl Trainer {
    pub fn new(config: Config, weights: &[f32], batch: usize) -> Result<Self, String> {
        pollster::block_on(Self::create(config, weights, batch))
    }
    async fn create(config: Config, weights: &[f32], batch: usize) -> Result<Self, String> {
        config.validate().map_err(str::to_string)?;
        if weights.len() != config.parameters()
            || weights.iter().any(|v| !v.is_finite())
            || !(1..=128).contains(&batch)
        {
            return Err("Tensor GPU incompatible".into());
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
            return Err("No se acepta un adaptador CPU como prueba de entrenamiento GPU".into());
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Boddy Tiny training"),
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        let sizes = [
            32,
            weights.len() * 4,
            batch * config.context * 4,
            batch * 4,
            batch * config.hidden * 4,
            batch * config.vocab * 4,
            weights.len() * 4,
            batch * 4,
        ];
        let tensor_bytes = sizes.iter().sum::<usize>() as u64;
        let mut buffers = Vec::new();
        for (index, size) in sizes.iter().enumerate() {
            let usage = if index == 0 {
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            } else {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
            };
            buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Boddy training tensor"),
                    contents: &vec![0u8; *size],
                    usage,
                }),
            );
        }
        queue.write_buffer(&buffers[1], 0, bytemuck::cast_slice(weights));
        let entries: Vec<_> = (0..8)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 0 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: matches!(binding, 2 | 3),
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Boddy tensor contract"),
            entries: &entries,
        });
        let bind_entries: Vec<_> = buffers
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
            entries: &bind_entries,
        });
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Boddy MLP from scratch"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mlp.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let forward = pipeline("forward");
        let gradient = pipeline("gradient");
        let update = pipeline("update");
        if let Some(error) = scope.pop().await {
            return Err(error.to_string());
        }
        Ok(Self {
            config,
            batch,
            device,
            queue,
            buffers,
            bind,
            forward,
            gradient,
            update,
            device_name: info.name,
            tensor_bytes,
        })
    }
    fn dispatch(
        &self,
        inputs: &[u32],
        targets: &[u32],
        rate: f32,
        update: bool,
    ) -> Result<(), String> {
        if inputs.len() != self.batch * self.config.context
            || targets.len() != self.batch
            || inputs
                .iter()
                .chain(targets)
                .any(|t| *t as usize >= self.config.vocab)
            || !rate.is_finite()
            || !(0.0..=1.0).contains(&rate)
        {
            return Err("Lote GPU o tasa invalida".into());
        }
        let uniform = [
            self.config.context as u32,
            self.config.vocab as u32,
            self.config.hidden as u32,
            self.batch as u32,
            self.config.parameters() as u32,
            rate.to_bits(),
            0,
            0,
        ];
        self.queue
            .write_buffer(&self.buffers[0], 0, bytemuck::cast_slice(&uniform));
        self.queue
            .write_buffer(&self.buffers[2], 0, bytemuck::cast_slice(inputs));
        self.queue
            .write_buffer(&self.buffers[3], 0, bytemuck::cast_slice(targets));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let passes = [
            (&self.forward, self.batch.div_ceil(32) as u32),
            (&self.gradient, self.config.parameters().div_ceil(64) as u32),
            (&self.update, self.config.parameters().div_ceil(64) as u32),
        ];
        for (pipeline, groups) in passes.iter().take(if update { 3 } else { 2 }) {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.dispatch_workgroups(*groups, 1, 1);
        }
        self.queue.submit([encoder.finish()]);
        Ok(())
    }
    fn read(&self, index: usize) -> Result<Vec<f32>, String> {
        let size = self.buffers[index].size();
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Boddy tensor readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.buffers[index], 0, &output, 0, size);
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        output
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
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
            return Err("Tensor GPU no finito".into());
        }
        Ok(values)
    }
    pub fn probe(&self, inputs: &[u32], targets: &[u32]) -> Result<(f32, Vec<f32>), String> {
        self.dispatch(inputs, targets, 0.0, false)?;
        let losses = self.read(7)?;
        let gradients = self.read(6)?;
        Ok((losses.iter().sum::<f32>() / self.batch as f32, gradients))
    }
    pub fn step(&self, inputs: &[u32], targets: &[u32], rate: f32) -> Result<(), String> {
        self.dispatch(inputs, targets, rate, true)
    }
    pub fn weights(&self) -> Result<Vec<f32>, String> {
        self.read(1)
    }
}
