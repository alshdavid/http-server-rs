mod templates;

use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Template {
  PreactRx,
  Preact,
  React,
  Vanilla,
}

impl Template {
  fn name(&self) -> &'static str {
    match self {
      Template::PreactRx => "preact-rx",
      Template::Preact => "preact",
      Template::React => "react",
      Template::Vanilla => "vanilla",
    }
  }
}

#[derive(Debug, Parser)]
pub struct GenerateCommand {
  /// Template project to create
  #[arg(long = "template", short = 't', value_enum)]
  pub template: Template,

  /// Directory to write the generated project to
  #[arg(long = "out-dir", short = 'o', default_value = "./template")]
  pub out_dir: PathBuf,
}

pub async fn main(args: GenerateCommand) -> anyhow::Result<()> {
  if args.out_dir.exists() {
    anyhow::bail!(
      "Output directory already exists: {}",
      args.out_dir.display()
    );
  }

  match args.template {
    Template::PreactRx => templates::preact_rx::generate(&args.out_dir)?,
    Template::Preact => templates::preact::generate(&args.out_dir)?,
    Template::React => templates::react::generate(&args.out_dir)?,
    Template::Vanilla => templates::vanilla::generate(&args.out_dir)?,
  }

  println!(
    "Created {} template in {}",
    args.template.name(),
    args.out_dir.display()
  );

  Ok(())
}
