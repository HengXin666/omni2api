# Omni2API installer for Windows: puts omni2api.exe and oh-seat.exe in ~\.omni2api\bin.
#   irm https://raw.githubusercontent.com/Juwan-Hwang/omni2api/main/install.ps1 | iex
# Then: omni2api up
$ErrorActionPreference = "Stop"
$repo = "Juwan-Hwang/omni2api"
$home2 = if ($env:OMNI2API_HOME) { $env:OMNI2API_HOME } else { Join-Path $HOME ".omni2api" }
$dir = Join-Path $home2 "bin"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
foreach ($b in "omni2api", "oh-seat") {
  Invoke-WebRequest -UseBasicParsing "https://github.com/$repo/releases/latest/download/$b-x86_64-pc-windows-msvc.exe" -OutFile (Join-Path $dir "$b.exe")
}
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not ($userPath -split ";" | Where-Object { $_ -eq $dir })) {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$dir", "User")
}
$env:Path = "$env:Path;$dir"
Write-Host "installed: $dir\omni2api.exe $dir\oh-seat.exe (added to user PATH)"
Write-Host "next: omni2api up"
