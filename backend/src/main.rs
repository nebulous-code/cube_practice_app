#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cube_backend::run().await
}
