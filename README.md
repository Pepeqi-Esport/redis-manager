# Redis Manager

Desktop app to start and stop a local Redis server interactively (egui).

The Redis Windows runtime is **embedded in the `.exe`**. On first launch it is extracted to `%LOCALAPPDATA%\RedisManager\runtime\redis`, so you only need to distribute a single binary.

## Requirements

* Rust stable (`rustup`) with `x86_64-pc-windows-gnu`
* MinGW-w64 (`%USERPROFILE%\.local\mingw64`) — used as the linker

## Build & run

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\.local\mingw64\bin;" + $env:Path
cd D:\Coding\Rust\redis-manager
cargo run --release
```

Release binary:

```powershell
cargo build --release
# output: target\release\redis-manager.exe
```

## Features

* Start / stop local Redis (`127.0.0.1:6379`)
* Live status indicator (running / not running)
* Start at login (Windows startup entry)
* Single-exe distribution (embedded Redis runtime + DLLs)
* App icon from `assets/images/logo`
