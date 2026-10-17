use crate::platform::self_upgrade;
use crate::platform::self_upgrade::TARGET_REPO;
use crate::platform::self_upgrade::UpgradeOptions;
use crate::platform::self_upgrade::UpgradeOutcome;

#[derive(Debug, clap::Parser)]
pub struct UpdateCommand {}

pub async fn main(_args: UpdateCommand) -> anyhow::Result<()> {
  let current_version = env!("CARGO_PKG_VERSION");

  println!("Starting Self Upgrade");

  let outcome = self_upgrade::try_upgrade(&UpgradeOptions {
    target_repo: TARGET_REPO,
    current_version,
  })
  .await;

  match outcome {
    Ok(UpgradeOutcome::Skip) => {
      println!("Already on the latest version ({})", current_version);
    }
    Ok(UpgradeOutcome::Success) => {
      println!("Upgrade complete");
    }
    Err(err) => {
      eprintln!("Upgrade failed: {err}");
    }
  }

  Ok(())
}
