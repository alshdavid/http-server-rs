#[derive(Debug, clap::Parser)]
pub struct UpdateCommand {}

pub async fn main(_args: UpdateCommand) -> anyhow::Result<()> {
  Ok(())
}
