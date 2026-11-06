struct Config { nodes:u32, parameters:u32, loss:u32, label:u32, context:u32, dimension:u32, rate:f32, pad:u32 }
@group(0) @binding(0) var<uniform> cfg:Config;
@group(0) @binding(1) var<storage,read> tape:array<vec4<u32>>;
@group(0) @binding(2) var<storage,read_write> weights:array<f32>;
@group(0) @binding(3) var<storage,read> inputs:array<u32>;
@group(0) @binding(4) var<storage,read_write> values:array<f32>;
@group(0) @binding(5) var<storage,read_write> grads:array<f32>;
// Interprete escalar de referencia: una invocacion, sin atomicos ni carreras.
@compute @workgroup_size(1)
fn execute() {
    for(var i=0u;i<cfg.nodes;i++) {
        let n=tape[i];let a=n.y;let b=n.z;var result=0.0;
        switch n.x {
            case 0u:{result=weights[a];}
            case 1u:{result=bitcast<f32>(a);}
            case 2u:{result=values[a]+values[b];}
            case 3u:{result=values[a]*values[b];}
            case 4u:{result=exp(values[a]);}
            case 5u:{result=log(values[a]);}
            case 6u:{result=inverseSqrt(values[a]);}
            case 7u:{result=tanh(values[a]);}
            case 8u:{result=max(values[a],values[b]);}
            case 9u:{result=weights[inputs[a]*cfg.dimension+b];}
            case 10u:{result=values[a+cfg.label];}
            default:{}
        }
        values[i]=result;grads[i]=0.0;
    }
    grads[cfg.loss]=1.0;
    for(var remaining=cfg.nodes;remaining>0u;remaining--) {
        let i=remaining-1u;let n=tape[i];let a=n.y;let b=n.z;let g=grads[i];
        switch n.x {
            case 2u:{grads[a]+=g;grads[b]+=g;}
            case 3u:{grads[a]+=g*values[b];grads[b]+=g*values[a];}
            case 4u:{grads[a]+=g*values[i];}
            case 5u:{grads[a]+=g/values[a];}
            case 6u:{grads[a]-=0.5*g*values[i]*values[i]*values[i];}
            case 7u:{grads[a]+=g*(1.0-values[i]*values[i]);}
            case 8u:{if values[a]>=values[b]{grads[a]+=g;}else{grads[b]+=g;}}
            case 9u:{grads[inputs[a]*cfg.dimension+b]+=g;}
            case 10u:{grads[a+cfg.label]+=g;}
            default:{}
        }
    }
    var norm=0.0;for(var p=0u;p<cfg.parameters;p++){norm+=grads[p]*grads[p];}
    let scale=cfg.rate/max(1.0,sqrt(norm));
    for(var p=0u;p<cfg.parameters;p++){weights[p]-=scale*grads[p];}
}
