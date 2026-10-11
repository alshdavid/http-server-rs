mod cmd;
mod platform;

use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
struct Command {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
  /// Serve a target directory over HTTP
  #[clap(name = "serve")]
  Serve(cmd::serve::ServeCommand),
  /// Update the binary to the latest release
  #[clap(name = "update")]
  Update(cmd::update::UpdateCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  match Command::parse().command {
    Commands::Serve(args) => cmd::serve::main(args).await,
    Commands::Update(args) => cmd::update::main(args).await,
  }
}
