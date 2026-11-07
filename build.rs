fn main() {
    // Embed the app icon into the Windows .exe (Explorer / taskbar / Alt-Tab).
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/images/logo/icon.ico");
        if let Err(err) = res.compile() {
            println!("cargo:warning=Failed to embed Windows icon: {err}");
        }
    }

    println!("cargo:rerun-if-changed=assets/images/logo/icon.ico");
    println!("cargo:rerun-if-changed=assets/images/logo/logo-256.png");
    // Rebuild when embedded Redis runtime files change.
    println!("cargo:rerun-if-changed=runtime/redis");
}
