//! The supported transparent compiler-cache composition, not binary attestation.
use std::path::Path;
#[cfg(not(test))]
use std::path::PathBuf;

pub const RELEASE_INPUT: &str = "observed-compiler-cache-release";
const SUPPORTED_RELEASE: &str = "sccache 0.18.0";

/// Like the compiler/dependency profile, this assumes the installed program and
/// local cache are trusted and unmodified. A similarly named wrapper is not a
/// supported composition. Workspace-wrapper nesting remains unsupported.
pub fn is_supported(program: &str, release: &str) -> bool {
    program.is_empty() || (named_sccache(Path::new(program)) && release == SUPPORTED_RELEASE)
}

/// Observe the configured program rather than accepting a release from the
/// environment. Replacing that executable or changing PATH reruns qualification.
#[cfg(not(test))]
pub fn observe(program: &str) -> String {
    if program.is_empty() {
        return String::new();
    }
    println!("cargo:rerun-if-env-changed=PATH");
    let Some(path) = resolve(program).filter(|path| named_sccache(path)) else {
        return String::new();
    };
    println!("cargo:rerun-if-changed={}", path.display());
    if let Ok(target) = path.canonicalize() {
        println!("cargo:rerun-if-changed={}", target.display());
    }
    match std::process::Command::new(path).arg("--version").output() {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_owned(),
        _ => String::new(),
    }
}

fn named_sccache(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("sccache" | "sccache.exe")
    )
}

#[cfg(not(test))]
fn resolve(program: &str) -> Option<PathBuf> {
    let path = Path::new(program);
    if path.is_absolute() || path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|directory| {
        let candidate = directory.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        if candidate.extension().is_none() {
            let executable = candidate.with_extension("exe");
            if executable.is_file() {
                return Some(executable);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_direct_or_the_observed_transparent_release_is_supported() {
        assert!(is_supported("", ""));
        for program in ["sccache", "sccache.exe", "C:/tools/sccache.exe"] {
            assert!(is_supported(program, "sccache 0.18.0"));
            assert!(!is_supported(program, ""));
            assert!(!is_supported(program, "sccache 0.17.0"));
        }
        for program in [
            "cache-wrapper.exe",
            "sccache-other.exe",
            "sccache.exe extra",
        ] {
            assert!(!is_supported(program, "sccache 0.18.0"));
        }
    }
}
