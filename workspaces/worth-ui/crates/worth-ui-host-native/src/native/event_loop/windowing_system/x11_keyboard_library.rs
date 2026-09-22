//! Locates `libxkbcommon-x11`, which winit's X11 backend loads with `dlopen`
//! and, when it is absent, panics over instead of returning an error
//! (`xkbcommon-dl-0.4.2/src/x11.rs:59`), together with `libxcb-xkb`, its one
//! `DT_NEEDED` dependency outside the base X11 client set: when that is absent
//! the `dlopen` fails and winit panics identically (measured: the library
//! present, its dependency missing, same panic). Locating both first turns
//! that crash into a typed denial before the event loop is built.
//!
//! The search mirrors the dynamic loader's order without reading its cache:
//! `LD_LIBRARY_PATH`, the directories `/etc/ld.so.conf` names (following its
//! `include` lines), then the loader's built-in directories. The cache is
//! generated from that same configuration, so this resolves against the set
//! the loader does.

use std::path::{Path, PathBuf};

/// The sonames winit's loader tries for `libxkbcommon-x11`, in its order.
const KEYBOARD_SONAMES: [&str; 2] = ["libxkbcommon-x11.so.0", "libxkbcommon-x11.so"];
/// The exact soname `libxkbcommon-x11`'s `DT_NEEDED` entry names.
const XCB_XKB_SONAMES: [&str; 1] = ["libxcb-xkb.so.1"];
const LOADER_CONFIGURATION: &str = "/etc/ld.so.conf";
const BUILT_IN_DIRECTORIES: [&str; 4] = ["/lib", "/usr/lib", "/lib64", "/usr/lib64"];
/// `ld.so.conf` includes `ld.so.conf.d/*.conf`; nothing deeper is configured
/// on any supported host, and the bound keeps a self-including file finite.
const INCLUDE_DEPTH: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct X11KeyboardLibraryAbsent;

/// Denies unless both libraries resolve somewhere in `search` order. They
/// need not share a directory; the loader resolves each independently.
pub(super) fn locate(search: &[PathBuf]) -> Result<(), X11KeyboardLibraryAbsent> {
    first_present(search, &KEYBOARD_SONAMES)?;
    first_present(search, &XCB_XKB_SONAMES)?;
    Ok(())
}

/// The first `sonames` candidate in `search` order that exists as a file.
fn first_present(
    search: &[PathBuf],
    sonames: &[&str],
) -> Result<PathBuf, X11KeyboardLibraryAbsent> {
    search
        .iter()
        .flat_map(|directory| sonames.iter().map(move |soname| directory.join(soname)))
        .find(|candidate| candidate.is_file())
        .ok_or(X11KeyboardLibraryAbsent)
}

/// The directories the dynamic loader resolves against on this host.
pub(super) fn loader_search_directories() -> Vec<PathBuf> {
    let mut directories = std::env::var_os("LD_LIBRARY_PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();
    directories.extend(configured_directories(
        Path::new(LOADER_CONFIGURATION),
        INCLUDE_DEPTH,
    ));
    directories.extend(BUILT_IN_DIRECTORIES.iter().map(PathBuf::from));
    directories
}

/// Directories named by one loader configuration file, following `include`
/// lines up to `depth`. An include whose final component is `*.conf` reads
/// every `.conf` file in that directory in sorted order — the one glob shape
/// the loader's own configuration uses.
fn configured_directories(configuration: &Path, depth: usize) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(configuration) else {
        return Vec::new();
    };
    let mut directories = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.strip_prefix("include ") {
            Some(pattern) if depth > 0 => {
                for included in included_configurations(Path::new(pattern.trim())) {
                    directories.extend(configured_directories(&included, depth - 1));
                }
            }
            Some(_) => {}
            None => directories.push(PathBuf::from(line)),
        }
    }
    directories
}

fn included_configurations(pattern: &Path) -> Vec<PathBuf> {
    let Some(parent) = pattern.parent() else {
        return Vec::new();
    };
    match pattern.file_name().and_then(|name| name.to_str()) {
        Some("*.conf") => {
            let Ok(entries) = std::fs::read_dir(parent) else {
                return Vec::new();
            };
            let mut included = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "conf")
                })
                .collect::<Vec<_>>();
            included.sort();
            included
        }
        Some(_) => vec![pattern.to_path_buf()],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "worth-ui-x11-keyboard-library-{name}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn first_present_takes_the_first_directory_holding_a_soname_and_denies_when_none_does() {
        let scratch = Scratch::new("first-present");
        let empty = scratch.0.join("empty");
        let versioned = scratch.0.join("versioned");
        let unversioned = scratch.0.join("unversioned");
        for directory in [&empty, &versioned, &unversioned] {
            std::fs::create_dir_all(directory).unwrap();
        }
        std::fs::write(versioned.join("libxkbcommon-x11.so.0"), b"").unwrap();
        std::fs::write(unversioned.join("libxkbcommon-x11.so"), b"").unwrap();
        assert_eq!(
            first_present(std::slice::from_ref(&empty), &KEYBOARD_SONAMES),
            Err(X11KeyboardLibraryAbsent),
            "an empty search denies"
        );
        assert_eq!(
            first_present(
                &[empty.clone(), unversioned.clone(), versioned.clone()],
                &KEYBOARD_SONAMES
            ),
            Ok(unversioned.join("libxkbcommon-x11.so")),
            "directory order wins over soname order"
        );
        assert_eq!(
            first_present(std::slice::from_ref(&versioned), &KEYBOARD_SONAMES),
            Ok(versioned.join("libxkbcommon-x11.so.0"))
        );
        assert_eq!(
            first_present(&[], &KEYBOARD_SONAMES),
            Err(X11KeyboardLibraryAbsent)
        );
    }

    #[test]
    fn locate_requires_the_keyboard_library_and_its_xcb_dependency_from_any_directories() {
        let scratch = Scratch::new("locate");
        let keyboard_only = scratch.0.join("keyboard-only");
        let dependency_only = scratch.0.join("dependency-only");
        for directory in [&keyboard_only, &dependency_only] {
            std::fs::create_dir_all(directory).unwrap();
        }
        std::fs::write(keyboard_only.join("libxkbcommon-x11.so.0"), b"").unwrap();
        std::fs::write(dependency_only.join("libxcb-xkb.so.1"), b"").unwrap();
        assert_eq!(
            locate(std::slice::from_ref(&keyboard_only)),
            Err(X11KeyboardLibraryAbsent),
            "the library alone still fails to load: its dependency is denied too"
        );
        assert_eq!(
            locate(std::slice::from_ref(&dependency_only)),
            Err(X11KeyboardLibraryAbsent)
        );
        assert_eq!(
            locate(&[keyboard_only.clone(), dependency_only.clone()]),
            Ok(()),
            "each library resolves independently along the search order"
        );
    }

    #[test]
    fn configured_directories_follow_includes_and_skip_comments_within_the_depth_bound() {
        let scratch = Scratch::new("configured");
        let conf_d = scratch.0.join("ld.so.conf.d");
        std::fs::create_dir_all(&conf_d).unwrap();
        let root_conf = scratch.0.join("ld.so.conf");
        std::fs::write(
            &root_conf,
            format!(
                "include {}/*.conf\n\n# comment\n/opt/first\n",
                conf_d.display()
            ),
        )
        .unwrap();
        std::fs::write(conf_d.join("b.conf"), "/opt/b\n").unwrap();
        std::fs::write(
            conf_d.join("a.conf"),
            format!("# libc\n/opt/a\ninclude {}\n", root_conf.display()),
        )
        .unwrap();
        std::fs::write(conf_d.join("ignored.txt"), "/opt/ignored\n").unwrap();
        let expected = ["/opt/a", "/opt/first", "/opt/b", "/opt/first"]
            .map(PathBuf::from)
            .to_vec();
        assert_eq!(configured_directories(&root_conf, INCLUDE_DEPTH), expected);
        assert_eq!(
            configured_directories(&root_conf, 0),
            vec![PathBuf::from("/opt/first")],
            "depth zero reads only the named file"
        );
        assert!(configured_directories(&scratch.0.join("absent.conf"), INCLUDE_DEPTH).is_empty());
    }
}
