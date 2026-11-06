#![no_std]
//! Decoder experimental de un bloque, una cabeza causal, RMSNorm y FFN tanh.
//! La cinta escalar es referencia verificable; no es un kernel optimizado.
extern crate alloc;
use alloc::{string::String, vec, vec::Vec};
pub const VOCAB: usize = 256;
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub context: usize,
    pub dimension: usize,
    pub feed_forward: usize,
}
impl Config {
    pub fn validate(self) -> Result<Self, &'static str> {
        if !(1..=16).contains(&self.context)
            || !(2..=16).contains(&self.dimension)
            || !(2..=32).contains(&self.feed_forward)
        {
            Err("Transformer fuera de limites")
        } else {
            Ok(self)
        }
    }
    pub fn parameters(self) -> usize {
        let d = self.dimension;
        2 * VOCAB * d + self.context * d + 4 * d * d + 2 * d * self.feed_forward + 3 * d
    }
}
pub fn encode(text: &str) -> Vec<u32> {
    text.bytes().map(u32::from).collect()
}
pub fn decode(tokens: &[u32]) -> Result<String, &'static str> {
    let bytes: Vec<u8> = tokens
        .iter()
        .map(|v| u8::try_from(*v).map_err(|_| "Token fuera de UTF8"))
        .collect::<Result<_, _>>()?;
    String::from_utf8(bytes).map_err(|_| "Secuencia UTF8 incompleta o invalida")
}
pub fn initialize(c: Config, seed: u32) -> Result<Vec<f32>, &'static str> {
    c.validate()?;
    let mut state = seed;
    let mut w = vec![0.0; c.parameters()];
    for v in &mut w {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        *v = ((state >> 8) as f32 / 16777216.0 - 0.5) * 0.2;
    }
    let norm = VOCAB * c.dimension + c.context * c.dimension;
    w[norm..norm + 3 * c.dimension].fill(1.0);
    Ok(w)
}
pub struct Graph {
    config: Config,
    nodes: Vec<[u32; 4]>,
    loss: usize,
    logits: Vec<usize>,
}
pub struct Evaluation {
    pub loss: f32,
    pub probabilities: Vec<f32>,
    pub gradient: Vec<f32>,
}
impl Graph {
    fn push(&mut self, op: u32, a: usize, b: usize) -> usize {
        let id = self.nodes.len();
        self.nodes.push([op, a as u32, b as u32, 0]);
        id
    }
    fn constant(&mut self, v: f32) -> usize {
        self.push(1, v.to_bits() as usize, 0)
    }
    fn sum(&mut self, values: &[usize]) -> usize {
        let mut v = self.constant(0.0);
        for x in values {
            v = self.push(2, v, *x);
        }
        v
    }
    fn linear(&mut self, input: &[usize], base: usize, out: usize) -> Vec<usize> {
        (0..out)
            .map(|k| {
                let products: Vec<_> = input
                    .iter()
                    .enumerate()
                    .map(|(i, x)| self.push(3, *x, base + k * input.len() + i))
                    .collect();
                self.sum(&products)
            })
            .collect()
    }
    fn norm(&mut self, x: &[usize], gain: usize) -> Vec<usize> {
        let squares: Vec<_> = x.iter().map(|v| self.push(3, *v, *v)).collect();
        let sum = self.sum(&squares);
        let inv = self.constant(1.0 / x.len() as f32);
        let mean = self.push(3, sum, inv);
        let eps = self.constant(1e-5);
        let mean = self.push(2, mean, eps);
        let scale = self.push(6, mean, 0);
        x.iter()
            .enumerate()
            .map(|(i, v)| {
                let n = self.push(3, *v, scale);
                self.push(3, n, gain + i)
            })
            .collect()
    }
    pub fn build(c: Config) -> Result<Self, &'static str> {
        Self::build_at(c, c.context.saturating_sub(1))
    }
    pub fn build_at(c: Config, position: usize) -> Result<Self, &'static str> {
        c.validate()?;
        if position >= c.context {
            return Err("Posicion fuera de contexto");
        }
        let d = c.dimension;
        let pos = VOCAB * d;
        let norm = pos + c.context * d;
        let q = norm + 3 * d;
        let k = q + d * d;
        let v = k + d * d;
        let o = v + d * d;
        let f1 = o + d * d;
        let f2 = f1 + d * c.feed_forward;
        let head = f2 + c.feed_forward * d;
        let mut g = Self {
            config: c,
            nodes: Vec::new(),
            loss: 0,
            logits: Vec::new(),
        };
        for i in 0..c.parameters() {
            g.push(0, i, 0);
        }
        let mut embeddings = Vec::new();
        let mut normalized = Vec::new();
        for p in 0..=position {
            let x: Vec<_> = (0..d)
                .map(|j| {
                    let e = g.push(9, p, j);
                    g.push(2, e, pos + p * d + j)
                })
                .collect();
            let n = g.norm(&x, norm);
            embeddings.push(x);
            normalized.push(n);
        }
        let query = g.linear(&normalized[position], q, d);
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for n in &normalized {
            keys.push(g.linear(n, k, d));
            values.push(g.linear(n, v, d));
        }
        let mut scores = Vec::new();
        let scale = g.constant(1.0 / libm::sqrtf(d as f32));
        for key in &keys {
            let products: Vec<_> = query
                .iter()
                .zip(key)
                .map(|(a, b)| g.push(3, *a, *b))
                .collect();
            let sum = g.sum(&products);
            scores.push(g.push(3, sum, scale));
        }
        let mut max = scores[0];
        for score in &scores[1..] {
            max = g.push(8, max, *score);
        }
        let negative = g.constant(-1.0);
        let negmax = g.push(3, max, negative);
        let mut exps = Vec::new();
        for score in &scores {
            let z = g.push(2, *score, negmax);
            exps.push(g.push(4, z, 0));
        }
        let sum = g.sum(&exps);
        let neglog = g.push(5, sum, 0);
        let neglog = g.push(3, neglog, negative);
        let inv = g.push(4, neglog, 0);
        let attention: Vec<_> = exps.iter().map(|e| g.push(3, *e, inv)).collect();
        let mixed: Vec<_> = (0..d)
            .map(|j| {
                let products: Vec<_> = attention
                    .iter()
                    .zip(&values)
                    .map(|(a, v)| g.push(3, *a, v[j]))
                    .collect();
                g.sum(&products)
            })
            .collect();
        let projected = g.linear(&mixed, o, d);
        let residual: Vec<_> = projected
            .iter()
            .zip(&embeddings[position])
            .map(|(a, b)| g.push(2, *a, *b))
            .collect();
        let normalized = g.norm(&residual, norm + d);
        let ff = g.linear(&normalized, f1, c.feed_forward);
        let ff: Vec<_> = ff.iter().map(|n| g.push(7, *n, 0)).collect();
        let ff = g.linear(&ff, f2, d);
        let residual: Vec<_> = ff
            .iter()
            .zip(&residual)
            .map(|(a, b)| g.push(2, *a, *b))
            .collect();
        let normalized = g.norm(&residual, norm + 2 * d);
        let logits = g.linear(&normalized, head, VOCAB);
        let zero = g.constant(0.0);
        let start = g.nodes.len();
        g.logits = logits.iter().map(|l| g.push(2, *l, zero)).collect();
        let mut max = g.logits[0];
        for i in 1..VOCAB {
            max = g.push(8, max, g.logits[i]);
        }
        let negmax = g.push(3, max, negative);
        let exps: Vec<_> = (0..VOCAB)
            .map(|i| {
                let z = g.push(2, g.logits[i], negmax);
                g.push(4, z, 0)
            })
            .collect();
        let sum = g.sum(&exps);
        let log = g.push(5, sum, 0);
        let total = g.push(2, log, max);
        let picked = g.push(10, start, 0);
        let neg = g.push(3, picked, negative);
        g.loss = g.push(2, total, neg);
        Ok(g)
    }
    pub fn config(&self) -> Config {
        self.config
    }
    pub fn instructions(&self) -> &[[u32; 4]] {
        &self.nodes
    }
    pub fn loss_index(&self) -> usize {
        self.loss
    }
    pub fn evaluate(
        &self,
        w: &[f32],
        input: &[u32],
        target: u32,
        backward: bool,
    ) -> Result<Evaluation, &'static str> {
        if w.len() != self.config.parameters()
            || w.iter().any(|v| !v.is_finite())
            || input.len() != self.config.context
            || input.iter().any(|v| *v >= 256)
            || target >= 256
        {
            return Err("Tensores Transformer invalidos");
        }
        let mut values = vec![0.0; self.nodes.len()];
        for (i, node) in self.nodes.iter().enumerate() {
            let a = node[1] as usize;
            let b = node[2] as usize;
            values[i] = match node[0] {
                0 => w[a],
                1 => f32::from_bits(node[1]),
                2 => values[a] + values[b],
                3 => values[a] * values[b],
                4 => libm::expf(values[a]),
                5 => libm::logf(values[a]),
                6 => 1.0 / libm::sqrtf(values[a]),
                7 => libm::tanhf(values[a]),
                8 => values[a].max(values[b]),
                9 => w[input[a] as usize * self.config.dimension + b],
                10 => values[a + target as usize],
                _ => return Err("Operacion invalida"),
            };
            if !values[i].is_finite() {
                return Err("Transformer no finito");
            }
        }
        let max = self
            .logits
            .iter()
            .map(|i| values[*i])
            .fold(f32::NEG_INFINITY, f32::max);
        let mut probabilities: Vec<_> = self
            .logits
            .iter()
            .map(|i| libm::expf(values[*i] - max))
            .collect();
        let sum = probabilities.iter().sum::<f32>();
        for p in &mut probabilities {
            *p /= sum;
        }
        let mut grad = vec![0.0; self.nodes.len()];
        if backward {
            grad[self.loss] = 1.0;
            for i in (0..self.nodes.len()).rev() {
                let [op, a, b, _] = self.nodes[i];
                let a = a as usize;
                let b = b as usize;
                let g = grad[i];
                match op {
                    0 | 1 => {}
                    2 => {
                        grad[a] += g;
                        grad[b] += g;
                    }
                    3 => {
                        grad[a] += g * values[b];
                        grad[b] += g * values[a];
                    }
                    4 => grad[a] += g * values[i],
                    5 => grad[a] += g / values[a],
                    6 => grad[a] -= 0.5 * g * values[i] * values[i] * values[i],
                    7 => grad[a] += g * (1.0 - values[i] * values[i]),
                    8 => {
                        let j = if values[a] >= values[b] { a } else { b };
                        grad[j] += g;
                    }
                    9 => grad[input[a] as usize * self.config.dimension + b] += g,
                    10 => grad[a + target as usize] += g,
                    _ => return Err("Gradiente invalido"),
                }
            }
        }
        grad.truncate(w.len());
        if grad.iter().any(|v| !v.is_finite()) {
            return Err("Gradiente no finito");
        }
        Ok(Evaluation {
            loss: values[self.loss],
            probabilities,
            gradient: grad,
        })
    }
}
pub fn sgd(weights: &mut [f32], gradient: &[f32], rate: f32) -> Result<(), &'static str> {
    if weights.len() != gradient.len()
        || weights.iter().any(|v| !v.is_finite())
        || !rate.is_finite()
        || !(0.0..=0.1).contains(&rate)
        || gradient.iter().any(|v| !v.is_finite())
    {
        return Err("SGD invalido");
    }
    let norm = libm::sqrtf(gradient.iter().map(|v| v * v).sum::<f32>());
    if !norm.is_finite() {
        return Err("Norma de gradiente fuera de rango");
    }
    let scale = rate / norm.max(1.0);
    for (w, g) in weights.iter_mut().zip(gradient) {
        *w -= scale * g;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        Config {
            context: 3,
            dimension: 4,
            feed_forward: 8,
        }
    }
    #[test]
    fn unicode_bytes_roundtrip() {
        let text = "Boddy: español, 中文 y 🧠";
        assert_eq!(decode(&encode(text)).unwrap(), text);
        assert!(decode(&[255]).is_err());
    }
    #[test]
    fn gradients_match_finite_differences() {
        let c = config();
        let graph = Graph::build(c).unwrap();
        let mut w = initialize(c, 1337).unwrap();
        let x = [98, 111, 100];
        let e = graph.evaluate(&w, &x, 100, true).unwrap();
        let n = 256 * c.dimension + c.context * c.dimension;
        for i in [
            98 * c.dimension,
            111 * c.dimension + 1,
            n,
            n + 3 * c.dimension,
            n + 7 * c.dimension * c.dimension,
            w.len() - 1,
        ] {
            let old = w[i];
            let eps = 0.001;
            w[i] = old + eps;
            let a = graph.evaluate(&w, &x, 100, false).unwrap().loss;
            w[i] = old - eps;
            let b = graph.evaluate(&w, &x, 100, false).unwrap().loss;
            w[i] = old;
            assert!(
                ((a - b) / (2.0 * eps) - e.gradient[i]).abs() < 0.003,
                "indice {i}"
            );
        }
    }
    #[test]
    fn attention_is_causal_and_learns() {
        let c = config();
        let graph = Graph::build_at(c, 1).unwrap();
        let mut w = initialize(c, 1337).unwrap();
        let before = graph.evaluate(&w, &[1, 2, 3], 4, true).unwrap();
        let changed = graph.evaluate(&w, &[1, 2, 250], 4, false).unwrap();
        assert_eq!(before.probabilities, changed.probabilities);
        for _ in 0..60 {
            let e = graph.evaluate(&w, &[1, 2, 3], 4, true).unwrap();
            sgd(&mut w, &e.gradient, 0.03).unwrap();
        }
        assert!(graph.evaluate(&w, &[1, 2, 3], 4, false).unwrap().loss < before.loss * 0.8);
        assert!(graph.evaluate(&w, &[999, 2, 3], 4, false).is_err());
    }
}
