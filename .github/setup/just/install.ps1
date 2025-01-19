if ( $env:JUST_VERSION -eq $null ) 
{
  $target_version = curl --silent "https://api.github.com/repos/casey/just/releases/latest" | jq -r '.tag_name'
} else {
  $target_version = $env:JUST_VERSION
}

if ( $env:OUT_DIR -eq $null ) 
{
  $out_dir = "${env:HOME}/.local/just"
} else {
  $out_dir = $env:OUT_DIR
}

$platform = switch -Wildcard ([System.Runtime.InteropServices.RuntimeInformation]::OSDescription) {
  "*Windows*" { "windows" }
  "*Linux*"   { "linux" }
  "*Darwin*"  { "macos" }
  Default     { "unknown" }
}

$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
  "X64"  { "amd64" }
  "X86"  { "x86" }
  "Arm"  { "arm" }
  "Arm64" { "arm64" }
  Default { "unknown" }
}

$url = switch ("${platform}-${arch}") {
  "linux-amd64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-unknown-linux-musl.tar.gz"}
  "linux-arm64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-unknown-linux-musl.tar.gz" }
  "macos-amd64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-apple-darwin.tar.gz"}
  "macos-arm64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-apple-darwin.tar.gz" }
  "windows-amd64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-pc-windows-msvc.zip" }
  "windows-arm64" { "https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-pc-windows-msvc.zip" }
  Default { $null }
}

if (Test-Path $out_dir) {
  Remove-Item -Recurse -Force $out_dir | Out-Null
  New-Item -ItemType "directory" -Force -Path $out_dir | Out-Null
} else {
  New-Item -ItemType "directory" -Force -Path $out_dir | Out-Null
}

Invoke-WebRequest "${url}" -OutFile "${out_dir}/just.zip"
Expand-Archive "${out_dir}/just.zip" -DestinationPath "${out_dir}"

$env:Path = "${out_dir};${env:Path}"

if ($env:GITHUB_PATH -ne $null) {
  Write-Output $out_dir >> $env:GITHUB_PATH
}

just --version