//! Embed a project into the runtime binary: `RGM_EMBED=path.gmproj cargo build -p gmruntime`.
//! Generates a static that the runtime uses when launched without arguments.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RGM_EMBED");
    let embed = env::var("RGM_EMBED").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    let code = if embed.is_empty() {
        "pub static EMBEDDED_PROJECT: Option<&[u8]> = None;\n".to_string()
    } else {
        let abs = resolve(&embed).unwrap_or_else(|| PathBuf::from(&embed));
        println!("cargo:rerun-if-changed={}", abs.display());
        format!(
            // include_bytes! resolves relative paths against OUT_DIR, so an absolute path is required
            "pub static EMBEDDED_PROJECT: Option<&[u8]> = Some(include_bytes!({:?}));\n",
            abs.canonicalize()
                .unwrap_or_else(|_| abs.clone())
                .display()
                .to_string()
        )
    };
    fs::write(out_dir.join("embedded_project.rs"), code).expect("write embedded_project.rs");
}

/// The build script runs with cwd = crate dir; accept paths relative to the
/// crate dir or to the workspace root.
fn resolve(p: &str) -> Option<PathBuf> {
    let given = Path::new(p);
    if given.exists() {
        return Some(given.to_path_buf());
    }
    let cwd = env::current_dir().ok()?;
    for base in [&cwd, &cwd.join("../..")] {
        let cand = base.join(p);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}
