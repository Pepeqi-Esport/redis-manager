# Redis Manager

Desktop app to start and stop a local Redis server interactively (egui).

The Redis Windows runtime is **embedded in the `.exe`**. On first launch it is extracted to `%LOCALAPPDATA%\RedisManager\runtime\redis`, so you only need to distribute a single binary.

## Requirements

* Rust stable (`rustup`)
* On Windows GNU toolchain: MinGW-w64 on `PATH` (also used by `build.rs` for the app icon via `windres`)

## Build & run

```powershell
cd D:\Coding\Rust\redis-manager
cargo run
```

Release build:

```powershell
cargo run --release
```

Binary output: `target/debug/redis-manager.exe` or `target/release/redis-manager.exe`.

## Features

* Start / stop local Redis (`127.0.0.1:6379`)
* Live status indicator (running / not running)
* Start at login (Windows startup entry)
* Single-exe distribution (embedded Redis runtime + DLLs)
* App icon from `assets/images/logo`
