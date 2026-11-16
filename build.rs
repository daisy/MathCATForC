//! Generate C/C++ headers (and optionally unpack Rules) for MathCATForC.
//!
//! All generation writes to OUT_DIR first so `cargo publish` is happy. When not
//! packaging for crates.io, outputs are also mirrored into `include/` (canonical
//! headers for the crate) and `Example/` (local/CI test directory).

extern crate cbindgen;

use cbindgen::*;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
extern crate embed_resource;

/// `cargo package` / `cargo publish` build under `target/package/...`.
fn is_packaging() -> bool {
    env::var("CARGO_MANIFEST_DIR")
        .map(|d| d.contains("target/package") || d.contains(r"target\package"))
        .unwrap_or(false)
}

fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

/// Mirror generated artifacts into the source tree for local builds and CI.
/// Skipped during `cargo publish` packaging so the source dir stays untouched.
fn mirror_for_local_use(out_dir: &Path, header_names: &[&str], rules_dir: Option<&Path>) {
    if is_packaging() {
        return;
    }
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = manifest_dir.join("include");
    let example_dir = manifest_dir.join("Example");
    fs::create_dir_all(&include_dir).expect("create include/");
    fs::create_dir_all(&example_dir).expect("create Example/");

    for name in header_names {
        let src = out_dir.join(name);
        copy_file(&src, &include_dir.join(name)).unwrap_or_else(|e| {
            panic!("copy {} to include/: {}", name, e);
        });
        copy_file(&src, &example_dir.join(name)).unwrap_or_else(|e| {
            panic!("copy {} to Example/: {}", name, e);
        });
    }

    if let Some(rules_src) = rules_dir {
        let example_rules = example_dir.join("Rules");
        if example_rules.exists() {
            let _ = fs::remove_dir_all(&example_rules);
        }
        copy_dir_all(rules_src, &example_rules).unwrap_or_else(|e| {
            panic!("copy Rules to Example/: {}", e);
        });
    }
}

fn write_headers(out_dir: &Path, name: &str, header_style: Language) {
    let config = Config::from_file(if header_style == Language::Cxx {
        "cbindgen.toml"
    } else {
        "cbindgen-c.toml"
    })
    .expect("couldn't read cbindgen config");
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest = out_dir.join(name);
    cbindgen::Builder::new()
        .with_language(header_style)
        .with_cpp_compat(true)
        .with_config(config)
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&dest);
}

#[cfg(feature = "include-zip")]
fn unzip_rules(location: &Path) {
    use zip::ZipArchive;

    let archive = libmathcat::ZIPPED_RULE_FILES;
    let archive = std::io::Cursor::new(archive);
    let mut zip_archive = ZipArchive::new(archive).unwrap();
    zip_archive.extract(location).expect("Zip extraction failed");
}

fn embed_description(out_dir: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _name: &str = option_env!("CARGO_PKG_NAME").unwrap_or("");
        let _version_major: &str = option_env!("CARGO_PKG_VERSION_MAJOR").unwrap_or("0");
        let _version_minor: &str = option_env!("CARGO_PKG_VERSION_MINOR").unwrap_or("0");
        let _version_patch: &str = option_env!("CARGO_PKG_VERSION_PATCH").unwrap_or("0");
        let _description: &str = option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("");

        let data = format!(
            "1 VERSIONINFO
FILEVERSION {_version_major},{_version_minor},{_version_patch}

BEGIN
	BLOCK \"StringFileInfo\"
	BEGIN
		BLOCK \"040904b0\"
		BEGIN
			VALUE \"FileDescription\", \"{_description}\"
			VALUE \"ProductName\", \"{_name}\"
			VALUE \"ProductVersion\", \"{_version_major}.{_version_minor}.{_version_patch}\"
		END
	END
	BLOCK \"VarFileInfo\"
	BEGIN
		VALUE \"Translation\", 0x409, 1200
	END
END"
        );
        let rc_path = out_dir.join("version.rc");
        fs::write(&rc_path, data).expect("Unable to write version.rc");
        embed_resource::compile(&rc_path, embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = out_dir;
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=cbindgen-c.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::create_dir_all(&out_dir).expect("create OUT_DIR");

    write_headers(&out_dir, "mathcat-c.h", Language::C);
    write_headers(&out_dir, "mathcat.h", Language::Cxx);

    #[cfg(feature = "include-zip")]
    let rules_dir = {
        // ZIP root is `Rules/`, so extract into OUT_DIR → OUT_DIR/Rules/...
        unzip_rules(&out_dir);
        Some(out_dir.join("Rules"))
    };
    #[cfg(not(feature = "include-zip"))]
    let rules_dir: Option<PathBuf> = None;

    mirror_for_local_use(
        &out_dir,
        &["mathcat-c.h", "mathcat.h"],
        rules_dir.as_deref(),
    );

    embed_description(&out_dir);
}
