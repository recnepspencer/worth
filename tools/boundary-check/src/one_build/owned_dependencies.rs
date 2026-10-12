//! Resolve governed package names, including renamed workspace dependencies.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Default)]
pub(super) struct OwnedDependencies {
    packages: BTreeSet<String>,
    workspaces: BTreeMap<String, BTreeMap<String, String>>,
}
impl OwnedDependencies {
    pub(super) fn from_manifests(manifests: &[(String, String)]) -> Self {
        let mut owners = Self::default();
        for (path, source) in manifests {
            let Ok(value) = toml::from_str::<toml::Value>(source) else {
                continue;
            };
            if let Some(name) = value
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
            {
                owners.packages.insert(name.to_owned());
            }
            if let Some(workspace) = value.get("workspace") {
                let directory = Path::new(path)
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let aliases = workspace
                    .get("dependencies")
                    .and_then(toml::Value::as_table)
                    .into_iter()
                    .flatten()
                    .map(|(name, dependency)| {
                        let package = dependency
                            .get("package")
                            .and_then(toml::Value::as_str)
                            .unwrap_or(name);
                        (name.clone(), package.to_owned())
                    })
                    .collect();
                owners.workspaces.insert(directory, aliases);
            }
        }
        owners
    }
    pub(super) fn owns(&self, path: &str, name: &str, dependency: &toml::Value) -> bool {
        if let Some(package) = dependency.get("package").and_then(toml::Value::as_str) {
            return self.packages.contains(package);
        }
        if dependency.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
            let mut directory = Path::new(path).parent();
            while let Some(current) = directory {
                let key = current.to_string_lossy().replace('\\', "/");
                if let Some(aliases) = self.workspaces.get(&key) {
                    return aliases
                        .get(name)
                        .is_some_and(|package| self.packages.contains(package));
                }
                directory = current.parent();
            }
        }
        self.packages.contains(name)
    }
}
