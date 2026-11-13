use std::process::ExitCode;

fn main() -> ExitCode {
    if let Some(code) = linn::runtime::bootstrap() {
        return ExitCode::from(code as u8);
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("async runtime")
        .block_on(run())
}
async fn run() -> ExitCode {
    match linn::cli::run().await {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("linn: {error}");
            ExitCode::from(2)
        }
    }
}
