use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};
#[allow(dead_code)] // The build uses only the compatibility decoder and asset writer.
#[path = "src/nlp/annotation.rs"]
mod nlp;
fn main() {
    println!("cargo:rerun-if-env-changed=LINN_MODEL");
    let source = PathBuf::from(
        env::var_os("LINN_MODEL")
            .expect("set LINN_MODEL to the local model directory at build time"),
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rerun-if-env-changed=LINN_POS_TOKENIZER_BUILD");
    let asset = PathBuf::from(
        env::var_os("LINN_POS_TOKENIZER_BUILD")
            .expect("supply pinned linguistic build input through the flake"),
    );
    let data = fs::read(&asset).expect("read linguistic build input");
    assert_eq!(
        format!("{:x}", Sha256::digest(&data)),
        "e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773"
    );
    println!("cargo:rerun-if-changed={}", asset.display());
    let tokenizer =
        nlp::Tokenizer::from_reader(data.as_slice()).expect("decode linguistic build input");
    fs::write(
        output.join("english.nlp"),
        tokenizer.compile_asset().expect("compile NLP asset"),
    )
    .expect("write lookup-ready linguistic asset");
    println!("cargo:rerun-if-changed=src/nlp");
    let mut fingerprint = Sha256::new();
    for name in ["config.json", "model.safetensors", "tokenizer.json"] {
        let path = source.join(name);
        fingerprint.update(name.as_bytes());
        let bytes = fs::read(&path).expect("read local model build input");
        let expected = match name {
            "model.safetensors" => {
                Some("473796430f4a5a042d05fd090ea7ff3986e942385d4cbea92fa82d79711a00c3")
            }
            "tokenizer.json" => {
                Some("d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66")
            }
            _ => None,
        };
        if let Some(expected) = expected {
            assert_eq!(
                format!("{:x}", Sha256::digest(&bytes)),
                expected,
                "unexpected pinned model asset {name}"
            );
        }
        fingerprint.update(bytes);
        println!("cargo:rerun-if-changed={}", path.display());
        let destination = output.join(name);
        if destination.exists() {
            fs::remove_file(&destination).expect("replace previous model build input");
        }
        fs::copy(path, destination).expect("copy local model into build output");
    }
    println!(
        "cargo:rustc-env=LINN_MODEL_FINGERPRINT={:x}",
        fingerprint.finalize()
    );
}
