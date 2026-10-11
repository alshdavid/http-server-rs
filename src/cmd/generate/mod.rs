mod templates;

use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Template {
  Preact,
  React,
  Vanilla,
}

impl Template {
  fn name(&self) -> &'static str {
    match self {
      Template::Preact => "preact",
      Template::React => "react",
      Template::Vanilla => "vanilla",
    }
  }
}

#[derive(Debug, Parser)]
pub struct GenerateCommand {
  /// Template project to create
  #[arg(long = "template", value_enum)]
  pub template: Template,

  /// Directory to write the generated project to
  #[arg(long = "out-dir", default_value = "./template")]
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
