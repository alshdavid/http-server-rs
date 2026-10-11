#[derive(Debug, clap::Parser)]
pub struct ServeCommand {}

pub async fn main(_args: ServeCommand) -> anyhow::Result<()> {
  Ok(())
}
