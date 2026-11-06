struct Config { context:u32, vocab:u32, hidden:u32, batch:u32, count:u32, rate:f32, pad0:u32, pad1:u32 }
@group(0) @binding(0) var<uniform> cfg:Config;
@group(0) @binding(1) var<storage,read_write> weights:array<f32>;
@group(0) @binding(2) var<storage,read> inputs:array<u32>;
@group(0) @binding(3) var<storage,read> targets:array<u32>;
@group(0) @binding(4) var<storage,read_write> hidden:array<f32>;
@group(0) @binding(5) var<storage,read_write> probs:array<f32>;
@group(0) @binding(6) var<storage,read_write> grads:array<f32>;
@group(0) @binding(7) var<storage,read_write> losses:array<f32>;

@compute @workgroup_size(32)
fn forward(@builtin(global_invocation_id) id:vec3<u32>) {
    let b=id.x;if b>=cfg.batch {return;}
    let width=cfg.context*cfg.vocab+1u;
    let first=cfg.hidden*width;
    for(var j=0u;j<cfg.hidden;j++) {
        var value=weights[j*width+width-1u];
        for(var c=0u;c<cfg.context;c++){value+=weights[j*width+c*cfg.vocab+inputs[b*cfg.context+c]];}
        hidden[b*cfg.hidden+j]=tanh(value);
    }
    var logits:array<f32,128>;
    var maximum=-1e30;
    for(var k=0u;k<cfg.vocab;k++) {
        let base=first+k*(cfg.hidden+1u);
        var value=weights[base+cfg.hidden];
        for(var j=0u;j<cfg.hidden;j++){value+=weights[base+j]*hidden[b*cfg.hidden+j];}
        logits[k]=value; maximum=max(maximum,value);
    }
    var sum=0.0;
    for(var k=0u;k<cfg.vocab;k++){logits[k]=exp(logits[k]-maximum);sum+=logits[k];}
    for(var k=0u;k<cfg.vocab;k++){probs[b*cfg.vocab+k]=logits[k]/sum;}
    losses[b]=-log(max(probs[b*cfg.vocab+targets[b]],1e-9));
}

@compute @workgroup_size(64)
fn gradient(@builtin(global_invocation_id) id:vec3<u32>) {
    let p=id.x;if p>=cfg.count {return;}
    let width=cfg.context*cfg.vocab+1u;
    let first=cfg.hidden*width;
    var value=0.0;
    if p>=first {
        let k=(p-first)/(cfg.hidden+1u);
        let j=(p-first)%(cfg.hidden+1u);
        for(var b=0u;b<cfg.batch;b++) {
            var delta=probs[b*cfg.vocab+k];if targets[b]==k {delta-=1.0;}
            if j<cfg.hidden {value+=delta*hidden[b*cfg.hidden+j];}else{value+=delta;}
        }
    } else {
        let j=p/width;
        let column=p%width;
        for(var b=0u;b<cfg.batch;b++) {
            if column<width-1u {
                let c=column/cfg.vocab;let token=column%cfg.vocab;
                if inputs[b*cfg.context+c]!=token {continue;}
            }
            var delta=0.0;
            for(var k=0u;k<cfg.vocab;k++) {
                var output=probs[b*cfg.vocab+k];if targets[b]==k {output-=1.0;}
                delta+=output*weights[first+k*(cfg.hidden+1u)+j];
            }
            let h=hidden[b*cfg.hidden+j];
            value+=delta*(1.0-h*h);
        }
    }
    grads[p]=value/f32(cfg.batch);
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id:vec3<u32>) {
    let p=id.x;if p<cfg.count {weights[p]-=cfg.rate*grads[p];}
}
