use octocrab::models::repos::Release;

pub async fn get_latest_releases(repo: &str) -> anyhow::Result<Release> {
  let Some((owner, name)) = repo.split_once('/') else {
    return Err(anyhow::anyhow!("Invalid repo name"));
  };

  let release = octocrab::instance()
    .repos(owner, name)
    .releases()
    .get_latest()
    .await?;

  Ok(release)
}

// pub async fn get_latest_prerelease(repo: &str) -> anyhow::Result<Version> {
//   let Some((owner, name)) = repo.split_once('/') else {
//     return Err(anyhow::anyhow!("Invalid repo name"));
//   };

//   let releases = octocrab::instance()
//     .repos(owner, name)
//     .releases()
//     .list()
//     .send()
//     .await?;

//   let release = releases
//     .items
//     .into_iter()
//     .find(|release| release.prerelease)
//     .ok_or_else(|| anyhow::anyhow!("no prerelease found for {repo}"))?;

//   normalize_tag(&release.tag_name)
// }

pub fn normalize_tag(tag: &str) -> anyhow::Result<semver::Version> {
  let tag = tag.strip_prefix('v').unwrap_or(tag);
  Ok(semver::Version::parse(tag)?)
}

pub fn resolve_asset(release: &Release) -> anyhow::Result<String> {
  let os = match std::env::consts::OS {
    "windows" => &["win32", "windows", "win"][..],
    "macos" => &["macos", "darwin"][..],
    "linux" => &["linux"][..],
    other => anyhow::bail!("unsupported operating system: {other}"),
  };

  let arch = match std::env::consts::ARCH {
    "x86_64" => &["amd64", "x86_64"][..],
    "aarch64" => &["arm64", "aarch64"][..],
    other => anyhow::bail!("unsupported cpu architecture: {other}"),
  };

  for asset in &release.assets {
    let name = asset.name.to_ascii_lowercase();
    if !name.ends_with(".tar.gz") {
      continue;
    }
    if os.iter().any(|s| name.contains(s)) && arch.iter().any(|s| name.contains(s)) {
      return Ok(asset.browser_download_url.to_string());
    }
  }

  anyhow::bail!(
    "no compatible asset found for {} {}",
    std::env::consts::OS,
    std::env::consts::ARCH
  )
}

pub async fn check_for_update(
  options: &UpgradeOptions<'_>
) -> anyhow::Result<Option<semver::Version>> {
  let current_version = semver::Version::parse(options.current_version)?;

  if !current_version.pre.is_empty() {
    return Ok(None);
  }

  let release = get_latest_releases(options.target_repo).await?;
  let remote_version = normalize_tag(&release.tag_name)?;

  if current_version < remote_version {
    return Ok(Some(remote_version));
  }

  Ok(None)
}

#[derive(Debug)]
pub enum UpgradeOutcome {
  Skip,
  Success,
}

#[derive(Debug)]
pub struct UpgradeOptions<'a> {
  pub target_repo: &'a str,
  pub current_version: &'a str,
}

pub async fn try_upgrade(options: &UpgradeOptions<'_>) -> anyhow::Result<UpgradeOutcome> {
  let current_version = semver::Version::parse(options.current_version)?;

  if !current_version.pre.is_empty() {
    return Ok(UpgradeOutcome::Skip);
  }

  let release = get_latest_releases(options.target_repo).await?;
  let remote_version = normalize_tag(&release.tag_name)?;

  if current_version >= remote_version {
    return Ok(UpgradeOutcome::Skip);
  }

  let asset = resolve_asset(&release)?;

  let mut rx = download_to_memory(asset.clone());
  let mut bytes = None;
  while let Some(progress) = rx.recv().await {
    match progress {
      DownloadProgress::Update(message) => println!("{message}"),
      DownloadProgress::Complete(buffer) => bytes = Some(buffer),
    }
  }

  let Some(bytes) = bytes else {
    return Err(anyhow::anyhow!("Download failed"));
  };

  let files = decompress_asset(&bytes)?;

  if files.is_empty() {
    return Err(anyhow::anyhow!("Malformed archive"));
  }

  let Some((_, file)) = files.first() else {
    return Err(anyhow::anyhow!("Malformed archive"));
  };

  replace_current_exe(file).await?;

  Ok(UpgradeOutcome::Success)
}

pub enum DownloadProgress {
  Update(String),
  Complete(Vec<u8>),
}

fn download_to_memory(url: String) -> tokio::sync::mpsc::UnboundedReceiver<DownloadProgress> {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

  tokio::spawn(async move {
    if let Err(err) = download_into_channel(&url, &tx).await {
      let _ = tx.send(DownloadProgress::Update(format!("download failed: {err}")));
    }
  });

  rx
}

async fn download_into_channel(
  url: &str,
  tx: &tokio::sync::mpsc::UnboundedSender<DownloadProgress>,
) -> anyhow::Result<()> {
  let mut res = reqwest::get(url).await?;

  if !res.status().is_success() {
    anyhow::bail!("download failed with status {}", res.status());
  }

  let total = res.content_length();
  let mut downloaded: u64 = 0;
  let mut reported: u64 = 0;
  let mut buffer: Vec<u8> = Vec::new();

  let _ = tx.send(DownloadProgress::Update("0%".to_string()));

  while let Some(chunk) = res.chunk().await? {
    buffer.extend_from_slice(&chunk);
    downloaded += chunk.len() as u64;

    if let Some(total) = total.filter(|total| *total > 0) {
      let step = (downloaded * 4 / total).min(4);
      if step > reported {
        reported = step;
        let _ = tx.send(DownloadProgress::Update(format!("{}%", step * 25)));
      }
    }
  }

  if reported < 4 {
    let _ = tx.send(DownloadProgress::Update("100%".to_string()));
  }

  let _ = tx.send(DownloadProgress::Complete(buffer));

  Ok(())
}

fn decompress_asset(asset: &[u8]) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
  let decoder = flate2::read::GzDecoder::new(asset);
  let mut archive = tar::Archive::new(decoder);

  let mut files = Vec::new();
  for entry in archive.entries()? {
    let mut entry = entry?;
    let path = entry.path()?.to_string_lossy().into_owned();
    let mut contents = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut contents)?;
    files.push((path, contents));
  }

  Ok(files)
}

async fn replace_current_exe(file: &[u8]) -> anyhow::Result<()> {
  let current_exe = std::env::current_exe()?;
  let new_exe = current_exe.with_extension("new");

  tokio::fs::write(&new_exe, file).await?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(&new_exe, std::fs::Permissions::from_mode(0o755)).await?;
  }

  std::fs::rename(&new_exe, &current_exe)?;

  Ok(())
}
