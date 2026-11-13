//! @ref LLP 0002#static-vectors
use crate::error::{Error, Result};
use tokenizers::Tokenizer;
#[repr(C, align(64))]
struct Aligned<const N: usize>([u8; N]);
const MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model.safetensors"));
static MODEL: Aligned<{ MODEL_BYTES.len() }> = Aligned(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/model.safetensors"
)));
#[cfg(target_os = "macos")]
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn vvtanhf(output: *mut f32, input: *const f32, count: *const i32);
}
fn vector_tanh(input: &[f32]) -> [f32; 512] {
    assert_eq!(input.len(), 512);
    let mut output = [0.; 512];
    #[cfg(target_os = "macos")]
    // Both buffers contain exactly 512 live elements, are disjoint, and outlive the call.
    unsafe {
        vvtanhf(output.as_mut_ptr(), input.as_ptr(), &512);
    }
    #[cfg(not(target_os = "macos"))]
    for (out, x) in output.iter_mut().zip(input) {
        *out = x.tanh();
    }
    output
}
pub struct Encoder {
    tokenizer: Tokenizer,
    weights: &'static [f32],
    alpha: &'static [f32],
    beta: &'static [f32],
    bias: &'static [f32],
}
impl Encoder {
    pub fn load() -> Result<Self> {
        let invalid = |e: String| Error::Invalid(format!("invalid embedded model: {e}"));
        let tensors =
            safetensors::SafeTensors::deserialize(&MODEL.0).map_err(|e| invalid(e.to_string()))?;
        let read = |name: &str, shape: &[usize]| -> Result<&'static [f32]> {
            let t = tensors.tensor(name).map_err(|e| invalid(e.to_string()))?;
            if t.dtype() != safetensors::Dtype::F32 || t.shape() != shape {
                return Err(invalid(format!("unexpected tensor {name}")));
            }
            bytemuck::try_cast_slice::<u8, f32>(t.data()).map_err(|e| invalid(e.to_string()))
        };
        Ok(Self {
            tokenizer: Tokenizer::from_bytes(include_bytes!(concat!(
                env!("OUT_DIR"),
                "/tokenizer.json"
            )))
            .map_err(|e| invalid(e.to_string()))?,
            weights: read("embedding.weight", &[30522, 512])?,
            alpha: read("dyt.alpha", &[512])?,
            beta: read("dyt.beta", &[512])?,
            bias: read("dyt.bias", &[512])?,
        })
    }
    fn ids(&self, text: &str) -> Vec<u32> {
        let text: String = text.chars().take(3584).collect();
        self.tokenizer
            .encode(text, false)
            .unwrap()
            .get_ids()
            .iter()
            .take(512)
            .copied()
            .collect()
    }
    pub fn encode(&self, text: &str) -> Vec<f32> {
        let ids = self.ids(text);
        let mut out = vec![0.; 512];
        if ids.is_empty() {
            return out;
        }
        for id in &ids {
            let row = &self.weights[*id as usize * 512..(*id as usize + 1) * 512];
            for (o, w) in out.iter_mut().zip(row) {
                *o += w;
            }
        }
        for (i, v) in out.iter_mut().enumerate() {
            *v = self.alpha[i] * (*v / ids.len() as f32) + self.bias[i];
        }
        let transformed = vector_tanh(&out);
        for (i, v) in out.iter_mut().enumerate() {
            *v = self.beta[i] * transformed[i];
        }
        let norm = out.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-9);
        for v in &mut out {
            *v /= norm;
        }
        out
    }
    pub fn pieces(&self, text: &str) -> Vec<String> {
        self.tokenizer
            .encode(text, false)
            .map(|e| {
                e.get_tokens()
                    .iter()
                    .filter_map(|s| {
                        let s = s.strip_prefix("##").unwrap_or(s);
                        (s.chars().all(char::is_alphabetic) && s.chars().count() > 2)
                            .then(|| s.to_owned())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn batch(&self, texts: &[String]) -> Vec<Vec<f32>> {
        let mut first = std::collections::HashMap::<&str, usize>::new();
        let mut vectors: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        for text in texts {
            let vector = if let Some(&index) = first.get(text.as_str()) {
                vectors[index].clone()
            } else {
                first.insert(text, vectors.len());
                self.encode(text)
            };
            vectors.push(vector);
        }
        vectors
    }
    pub fn similarity(a: &[f32], b: &[f32]) -> f32 {
        dot(a, b)
    }
}
#[cfg(target_os = "macos")]
pub const ARTIFACT_MODEL: &str =
    concat!(env!("LINN_MODEL_FINGERPRINT"), ":sse-v2-native-vforce-v1");
#[cfg(not(target_os = "macos"))]
pub const ARTIFACT_MODEL: &str =
    concat!(env!("LINN_MODEL_FINGERPRINT"), ":sse-v2-native-scalar-v1");
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    crate::retrieval::similarity::dot(a, b)
}
pub fn unit(mut v: Vec<f32>) -> Vec<f32> {
    let norm = dot(&v, &v).sqrt().max(1e-9);
    for x in &mut v {
        *x /= norm;
    }
    v
}
// @ref LLP 0007#store-publication-and-synchronization
pub(crate) fn shared_encoder() -> Result<std::sync::Arc<Encoder>> {
    static ENCODER: once_cell::sync::OnceCell<std::sync::Arc<Encoder>> =
        once_cell::sync::OnceCell::new();
    ENCODER
        .get_or_try_init(|| Encoder::load().map(std::sync::Arc::new))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_batch_text_keeps_each_original_float32_vector() {
        let encoder = Encoder::load().unwrap();
        let texts: Vec<_> = [
            "",
            "cache policy",
            "café 東京",
            "cache policy",
            "",
            "alpha-beta",
            "café 東京",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let actual = encoder.batch(&texts);
        for (text, vector) in texts.iter().zip(actual) {
            assert_eq!(
                vector.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                encoder
                    .encode(text)
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn native_mean_and_transform_match_independent_f64_equation() {
        let m = Encoder::load().unwrap();
        assert!(m.encode("").iter().all(|v| *v == 0.));
        for text in [
            "cache invalidation policy",
            "Rust Nix build tools",
            "🦄 résumé 東京",
            "alpha alpha beta",
        ] {
            let ids = m.ids(text);
            let mut expected: Vec<f64> = (0..512)
                .map(|i| {
                    let mean = ids
                        .iter()
                        .map(|id| m.weights[*id as usize * 512 + i] as f64)
                        .sum::<f64>()
                        / ids.len() as f64;
                    m.beta[i] as f64 * (m.alpha[i] as f64 * mean + m.bias[i] as f64).tanh()
                })
                .collect();
            let norm = expected.iter().map(|v| v * v).sum::<f64>().sqrt();
            for v in &mut expected {
                *v /= norm;
            }
            let actual = m.encode(text);
            assert!(
                actual
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| (*a as f64 - b).abs() < 2e-6)
            );
        }
        assert_eq!(m.ids(&"alpha ".repeat(600)).len(), 512);
    }
}
