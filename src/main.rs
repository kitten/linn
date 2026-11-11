use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match linn::cli::run().await {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("linn: {error}");
            ExitCode::from(2)
        }
    }
}
