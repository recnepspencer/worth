//! Registered type identities that host or drive work; ownership alone is irrelevant.
use super::construction_factories::Factories;
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

pub(super) const CONFIG: &str = "tools/boundary-check/config/runtime-roots.toml";
#[derive(Deserialize)]
struct Configuration {
    roots: Vec<RuntimeRoot>,
}
#[derive(Deserialize)]
struct RuntimeRoot {
    #[serde(rename = "type")]
    identity: String,
    reason: String,
    #[serde(default)]
    owner: String,
}
pub(super) struct Registry {
    identities: BTreeSet<String>,
    pub(super) factories: Factories,
}
impl Registry {
    pub(super) fn load(root: &Path) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(root.join(CONFIG)).map_err(|e| format!("{CONFIG}: {e}"))?;
        let config: Configuration = toml::from_str(&text).map_err(|e| format!("{CONFIG}: {e}"))?;
        let mut identities = BTreeSet::new();
        let mut owners = BTreeSet::new();
        for entry in config.roots {
            let path: syn::Path = syn::parse_str(&entry.identity)
                .map_err(|e| format!("{CONFIG}: invalid root type {}: {e}", entry.identity))?;
            if path.segments.is_empty()
                || entry.reason.trim().is_empty()
                || entry.reason.contains('\n')
            {
                return Err(format!(
                    "{CONFIG}: each runtime root needs a type and one-line reason"
                ));
            }
            if !identities.insert(entry.identity.clone()) {
                return Err(format!(
                    "{CONFIG}: duplicate runtime root {}",
                    entry.identity
                ));
            }
            if !entry.owner.is_empty() {
                if Path::new(&entry.owner).components().any(|part| {
                    !matches!(
                        part,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )
                }) {
                    return Err(format!("{CONFIG}: root owner must be repository-relative"));
                }
                owners.insert(entry.owner);
            }
        }
        let factories = Factories::derive(root, &owners)?;
        Ok(Self {
            identities,
            factories,
        })
    }
    pub(super) fn matching(&self, spelling: &str) -> Option<&str> {
        let parts: Vec<_> = spelling.split("::").collect();
        self.identities
            .iter()
            .find(|identity| {
                let expected: Vec<_> = identity.split("::").collect();
                parts.contains(expected.last().unwrap())
                    && (expected.len() == 1 || parts.first() == expected.first())
            })
            .map(String::as_str)
    }
}
