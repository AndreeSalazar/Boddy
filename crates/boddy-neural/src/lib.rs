#![no_std]
//! Boddy Tiny: contexto one-hot -> capa tanh -> softmax de siguiente carácter.
extern crate alloc;
use alloc::{vec, vec::Vec};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub context: usize,
    pub vocab: usize,
    pub hidden: usize,
}
impl Config {
    pub fn validate(self) -> Result<Self, &'static str> {
        if !(1..=16).contains(&self.context)
            || !(2..=128).contains(&self.vocab)
            || !(1..=64).contains(&self.hidden)
        {
            return Err("Arquitectura fuera de limites");
        }
        Ok(self)
    }
    pub fn first_layer(self) -> usize {
        self.hidden * (self.context * self.vocab + 1)
    }
    pub fn parameters(self) -> usize {
        self.first_layer() + self.vocab * (self.hidden + 1)
    }
}

pub struct Rng(pub u32);
impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }
    pub fn unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16777216.0
    }
}

pub fn initialize(config: Config, seed: u32) -> Result<Vec<f32>, &'static str> {
    config.validate()?;
    let mut rng = Rng(seed);
    let mut weights = vec![0.0; config.parameters()];
    for (i, w) in weights.iter_mut().enumerate() {
        let bias = if i < config.first_layer() {
            i % (config.context * config.vocab + 1) == config.context * config.vocab
        } else {
            (i - config.first_layer()) % (config.hidden + 1) == config.hidden
        };
        if !bias {
            *w = (rng.unit() - 0.5) * 0.2;
        }
    }
    Ok(weights)
}

fn check(config: Config, weights: &[f32], input: &[u32]) -> Result<(), &'static str> {
    config.validate()?;
    if weights.len() != config.parameters()
        || input.len() != config.context
        || input.iter().any(|t| *t as usize >= config.vocab)
    {
        return Err("Tensor incompatible");
    }
    Ok(())
}

pub fn forward(
    config: Config,
    weights: &[f32],
    input: &[u32],
) -> Result<(Vec<f32>, Vec<f32>), &'static str> {
    check(config, weights, input)?;
    let width = config.context * config.vocab + 1;
    let mut hidden = vec![0.0; config.hidden];
    for (j, h) in hidden.iter_mut().enumerate() {
        let mut value = weights[j * width + width - 1];
        for (c, token) in input.iter().enumerate() {
            value += weights[j * width + c * config.vocab + *token as usize];
        }
        *h = libm::tanhf(value);
    }
    let mut probabilities = vec![0.0; config.vocab];
    for (k, p) in probabilities.iter_mut().enumerate() {
        let base = config.first_layer() + k * (config.hidden + 1);
        *p = weights[base + config.hidden];
        for (j, h) in hidden.iter().enumerate() {
            *p += weights[base + j] * h;
        }
    }
    let max = probabilities
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0;
    for p in &mut probabilities {
        *p = libm::expf(*p - max);
        sum += *p;
    }
    for p in &mut probabilities {
        *p /= sum;
    }
    Ok((hidden, probabilities))
}

pub fn loss_and_gradient(
    config: Config,
    weights: &[f32],
    inputs: &[u32],
    targets: &[u32],
) -> Result<(f32, Vec<f32>), &'static str> {
    config.validate()?;
    let batch = targets.len();
    if batch == 0
        || batch > 128
        || inputs.len() != batch * config.context
        || targets.iter().any(|t| *t as usize >= config.vocab)
    {
        return Err("Lote incompatible");
    }
    let mut gradient = vec![0.0; config.parameters()];
    let mut loss = 0.0;
    let scale = 1.0 / batch as f32;
    for (sample, target) in targets.iter().enumerate() {
        let input = &inputs[sample * config.context..(sample + 1) * config.context];
        let (hidden, mut delta) = forward(config, weights, input)?;
        loss -= libm::logf(delta[*target as usize].max(1e-9)) * scale;
        delta[*target as usize] -= 1.0;
        for (k, d) in delta.iter().enumerate() {
            let base = config.first_layer() + k * (config.hidden + 1);
            for (j, h) in hidden.iter().enumerate() {
                gradient[base + j] += d * h * scale;
            }
            gradient[base + config.hidden] += d * scale;
        }
        for (j, h) in hidden.iter().enumerate() {
            let mut dh = 0.0;
            for (k, d) in delta.iter().enumerate() {
                dh += d * weights[config.first_layer() + k * (config.hidden + 1) + j];
            }
            dh *= (1.0 - h * h) * scale;
            let base = j * (config.context * config.vocab + 1);
            for (c, token) in input.iter().enumerate() {
                gradient[base + c * config.vocab + *token as usize] += dh;
            }
            gradient[base + config.context * config.vocab] += dh;
        }
    }
    Ok((loss, gradient))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gradient_matches_finite_differences() {
        let c = Config {
            context: 2,
            vocab: 3,
            hidden: 4,
        };
        let mut w = initialize(c, 7).unwrap();
        let x = [0, 1, 1, 2];
        let y = [2, 0];
        let (_, g) = loss_and_gradient(c, &w, &x, &y).unwrap();
        for i in [0, 1, 6, 12, c.first_layer(), c.parameters() - 1] {
            let original = w[i];
            let eps = 0.001;
            w[i] = original + eps;
            let plus = loss_and_gradient(c, &w, &x, &y).unwrap().0;
            w[i] = original - eps;
            let minus = loss_and_gradient(c, &w, &x, &y).unwrap().0;
            w[i] = original;
            assert!(
                ((plus - minus) / (2.0 * eps) - g[i]).abs() < 0.001,
                "param {i}"
            );
        }
    }
    #[test]
    fn learning_reduces_loss_and_invalid_tensors_are_rejected() {
        let c = Config {
            context: 2,
            vocab: 3,
            hidden: 4,
        };
        let mut w = initialize(c, 7).unwrap();
        let x = [0, 1];
        let y = [2];
        let before = loss_and_gradient(c, &w, &x, &y).unwrap().0;
        for _ in 0..100 {
            let (_, g) = loss_and_gradient(c, &w, &x, &y).unwrap();
            for (i, d) in g.iter().enumerate() {
                w[i] -= 0.2 * d;
            }
        }
        let after = loss_and_gradient(c, &w, &x, &y).unwrap().0;
        assert!(after < before * 0.2);
        assert!(forward(c, &w, &[99, 0]).is_err());
        assert!(Config {
            context: 0,
            vocab: 2,
            hidden: 1
        }
        .validate()
        .is_err());
    }
}
