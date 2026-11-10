//! Derive logo-256.png + embed Windows .exe icon from assets/images/logo/logo.png.
//!
//! On the GNU toolchain, rustc's `--gc-sections` drops unreferenced
//! resource objects unless we force `--whole-archive`.

fn main() {
    println!("cargo:rerun-if-changed=assets/images/logo/logo.png");
    // Rebuild when embedded Redis runtime files change.
    println!("cargo:rerun-if-changed=runtime/redis");

    if let Err(e) = generate_logo_256() {
        panic!("failed to generate logo-256.png: {e}");
    }

    #[cfg(windows)]
    {
        if let Err(e) = embed_windows_icon() {
            panic!("failed to embed Windows app icon: {e}");
        }
    }
}

/// Resize the master logo into the 256² asset used by `logo_helper`.
fn generate_logo_256() -> Result<(), Box<dyn std::error::Error>> {
    const SRC: &str = "assets/images/logo/logo.png";
    const DST: &str = "assets/images/logo/logo-256.png";

    let src = image::open(SRC)?.to_rgba8();
    let resized =
        image::imageops::resize(&src, 256, 256, image::imageops::FilterType::Lanczos3);
    resized.save(DST)?;
    println!("cargo:rerun-if-changed={DST}");
    Ok(())
}

#[cfg(windows)]
fn embed_windows_icon() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let ico_path = out.join("app_icon.ico");
    write_ico("assets/images/logo/logo.png", &ico_path)?;

    let mingw_bin = mingw_bin_dir();
    let mut res = winres::WindowsResource::new();
    if let Some(bin) = mingw_bin.as_ref() {
        let windres = bin.join("windres.exe");
        let ar = bin.join("ar.exe");
        if windres.exists() {
            res.set_windres_path(windres.to_str().unwrap());
        }
        if ar.exists() {
            res.set_ar_path(ar.to_str().unwrap());
        }
    }
    res.set_icon(ico_path.to_str().unwrap());
    res.set("ProductName", "Redis Manager");
    res.set(
        "FileDescription",
        "Redis Manager - interactive local Redis control",
    );
    res.compile()?;

    // Force the resource archive into the final .exe (gnu + gc-sections).
    let lib = out.join("libresource.a");
    println!("cargo:rustc-link-arg=-Wl,--whole-archive");
    println!("cargo:rustc-link-arg={}", lib.display());
    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    Ok(())
}

#[cfg(windows)]
fn mingw_bin_dir() -> Option<std::path::PathBuf> {
    if let Ok(windres) = std::env::var("WINDRES") {
        return std::path::PathBuf::from(windres)
            .parent()
            .map(|p| p.to_path_buf());
    }
    let home = std::env::var("USERPROFILE").ok()?;
    let candidate = std::path::PathBuf::from(home)
        .join(".local")
        .join("mingw64")
        .join("bin");
    candidate.exists().then_some(candidate)
}

#[cfg(windows)]
fn write_ico(png_path: &str, ico_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let src = image::open(png_path)?.to_rgba8();
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16_u32, 32, 48, 256] {
        let resized =
            image::imageops::resize(&src, size, size, image::imageops::FilterType::Lanczos3);
        let entry = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        dir.add_entry(ico::IconDirEntry::encode(&entry)?);
    }
    let mut file = std::fs::File::create(ico_path)?;
    dir.write(&mut file)?;
    Ok(())
}
