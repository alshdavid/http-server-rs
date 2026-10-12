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
  #[clap(name = "serve", visible_aliases = ["s"])]
  Serve(cmd::serve::ServeCommand),
  /// Generate a template project
  #[clap(name = "generate", visible_aliases = ["gen", "g"])]
  Generate(cmd::generate::GenerateCommand),
  /// Update the binary to the latest release
  #[clap(name = "update", visible_aliases = ["up", "u"])]
  Update(cmd::update::UpdateCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  match Command::parse().command {
    Commands::Generate(args) => cmd::generate::main(args).await,
    Commands::Serve(args) => cmd::serve::main(args).await,
    Commands::Update(args) => cmd::update::main(args).await,
  }
}
