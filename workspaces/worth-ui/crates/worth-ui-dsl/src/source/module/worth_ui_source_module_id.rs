use std::fmt;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorthUiSourceModuleId {
    canonical_text: String,
}

impl WorthUiSourceModuleId {
    pub(crate) fn from_workspace_path(
        workspace_root: &Path,
        module_path: &Path,
    ) -> Result<Self, String> {
        let relative = if module_path.is_absolute() {
            module_path
                .strip_prefix(workspace_root)
                .map_err(|_| "absolute module path must stay within workspace root".to_owned())?
                .to_path_buf()
        } else {
            module_path.to_path_buf()
        };
        Self::from_relative_path(&relative)
    }

    pub fn from_relative_path(relative_path: &Path) -> Result<Self, String> {
        let canonical_path = canonicalize_relative_path(relative_path)?;
        Ok(Self {
            canonical_text: canonical_path.to_string_lossy().replace('\\', "/"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.canonical_text
    }
}

impl fmt::Display for WorthUiSourceModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical_text)
    }
}

fn canonicalize_relative_path(relative_path: &Path) -> Result<PathBuf, String> {
    // Module paths may be spelled with either separator on any host. Normalize
    // before walking: on Linux `\` is an ordinary file-name byte, so without
    // this `.\app\main.wui` keeps its `CurDir` and canonicalizes to
    // `./app/main.wui`, a second identity for the same module.
    let portable = relative_path.to_string_lossy().replace('\\', "/");
    let mut normalized = PathBuf::new();
    for component in Path::new(&portable).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("module path cannot escape the package root".to_owned());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("module identity requires a workspace-relative path".to_owned());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("module path cannot be empty after canonicalization".to_owned());
    }

    Ok(normalized)
}
