set windows-shell := ["pwsh", "-NoLogo", "-NoProfileLoadTime", "-Command"]

profile := env_var_or_default("profile", "debug")

os := \
if \
  env_var_or_default("os", "") == "Windows_NT" { "windows" } \
else if \
  env_var_or_default("os", "") != "" { env_var("os") } \
else \
  { os() }

arch := \
if \
  env_var_or_default("arch", "") != "" { env_var("arch") } \
else if \
  arch() == "x86_64" { "amd64" } \
else if \
  arch() == "aarch64" { "arm64" } \
else \
  { arch() }

target := \
if \
  os + arch == "linuxamd64" { "x86_64-unknown-linux-gnu" } \
else if \
  os + arch == "linuxarm64" { "aarch64-unknown-linux-gnu" } \
else if \
  os + arch == "macosamd64" { "x86_64-apple-darwin" } \
else if\
  os + arch == "macosarm64" { "aarch64-apple-darwin" } \
else if \
  os + arch == "windowsamd64" { "x86_64-pc-windows-msvc" } \
else if \
  os + arch == "windowsarm64" { "aarch64-pc-windows-msvc" } \
else \
  { env_var_or_default("target", "debug") }

profile_cargo := \
if \
  profile != "debug" { "--profile " + profile } \
else \
  { "" }

target_cargo := \
if \
  target == "debug" { "" } \
else if \
  target == "" { "" } \
else \
  { "--target " + target } 

out_dir :=  join(justfile_directory(), "target", os + "-" + arch, profile)
out_dir_link :=  join(justfile_directory(), "target", profile)

[unix]
build:
  @rm -rf "{{out_dir}}"
  @rm -rf "{{out_dir_link}}"
  @mkdir -p "{{out_dir}}"
  cargo build {{profile_cargo}} {{target_cargo}}
  @cp "./target/.cargo/{{target}}/{{profile}}/http-server" "{{out_dir}}"
  @# ln -rs "{{out_dir}}" "{{out_dir_link}}"

[windows]
build:
  @if (Test-Path {{out_dir}}) { Remove-Item -Recurse -Force {{out_dir}} | Out-Null }
  @if (Test-Path {{out_dir_link}}) { Remove-Item -Recurse -Force {{out_dir_link}} | Out-Null }
  @New-Item -ItemType "directory" -Force -Path "{{out_dir}}"  | Out-Null
  cargo build {{profile_cargo}} {{target_cargo}}
  Copy-Item ".\target\.cargo\{{target}}\{{profile}}\http-server.exe" -Destination "{{out_dir}}" | Out-Null
  @# New-Item -Path "{{out_dir}}" -ItemType SymbolicLink -Value "{{out_dir_link}}"

[unix]
run *ARGS:
  just build
  {{out_dir}}/http-server {{ARGS}}

[windows]
run *ARGS:
  just build
  {{out_dir}}/http-server.exe {{ARGS}}

test:
  cargo test

fmt:
  cargo +nightly fmt
