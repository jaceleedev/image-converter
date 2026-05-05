#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    image_converter::server::run().await
}
