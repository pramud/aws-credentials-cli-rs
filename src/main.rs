use std::process;

#[tokio::main]
async fn main() {
    if let Err(error) = aws_credentials_cli::run().await {
        eprintln!("Application error: {error}");
        process::exit(1);
    }
}
