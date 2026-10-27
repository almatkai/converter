#[tokio::main(worker_threads = 2)]
async fn main() {
    if let Err(error) = tezbar_extension_sdk::serve(converter::Converter::default()).await {
        eprintln!("converter: {error}");
        std::process::exit(1);
    }
}
