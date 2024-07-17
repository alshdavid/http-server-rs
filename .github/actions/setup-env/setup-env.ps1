# Project Root
$env:RootPath = "$PSScriptRoot\..\..\.."

# Just
$env:Path = "${HOME}\.local\just;${env:Path}"

# Rust
$env:RUSTUP_HOME = "${HOME}\.local\rust\rustup"
$env:CARGO_HOME = "${HOME}\.local\rust\cargo"
$env:Path = "${env:USERPROFILE}\.cargo\bin;${env:Path}"

if (Test-Path "${env:RootPath}\.env") {
  echo "Loading .env"
  Get-Content "${env:RootPath}\.env" | foreach {
    $name, $value = $_.split('=')
    if ([string]::IsNullOrWhiteSpace($name) -and $name.Contains('#')) {
      continue
    }
    echo "Setting $name = $value"
    Set-Item "env:$name" "$value"
  }
}
