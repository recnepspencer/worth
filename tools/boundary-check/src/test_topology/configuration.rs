//! Governed workspace and closed executable-exception declarations.
use super::{CONFIG, EXCEPTIONS};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

pub(super) struct Configuration {
    pub(super) workspaces: Vec<Workspace>,
    pub(super) exceptions: BTreeSet<(String, String)>,
}
pub(super) struct Workspace {
    pub(super) path: String,
    pub(super) stem: String,
}

fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

pub(super) fn load(root: &Path) -> Result<Option<Configuration>, String> {
    if !root.join(CONFIG).exists() && !root.join(EXCEPTIONS).exists() {
        if root.join(".git").exists() {
            return Err("repository checkout is missing its test-topology configuration".into());
        }
        return Ok(None);
    }
    let workspaces =
        std::fs::read_to_string(root.join(CONFIG)).map_err(|error| error.to_string())?;
    let exceptions =
        std::fs::read_to_string(root.join(EXCEPTIONS)).map_err(|error| error.to_string())?;
    let workspaces: Vec<_> = lines(&workspaces)
        .map(|line| {
            let (path, stem) = line
                .split_once('|')
                .ok_or_else(|| format!("workspace needs path | court stem: {line}"))?;
            if path.trim().is_empty() || stem.trim().is_empty() {
                return Err(format!("workspace needs path | court stem: {line}"));
            }
            Ok(Workspace {
                path: path.trim().into(),
                stem: stem.trim().into(),
            })
        })
        .collect::<Result<_, String>>()?;
    for workspace in &workspaces {
        if Path::new(&workspace.path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err(format!(
                "governed workspace must be repository-relative: {}",
                workspace.path
            ));
        }
    }
    let mut declared = BTreeSet::new();
    for line in lines(&exceptions) {
        let parts: Vec<_> = line.split('|').map(str::trim).collect();
        if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
            return Err(format!(
                "exception needs manifest | target | reason: {line}"
            ));
        }
        if !declared.insert((parts[0].into(), parts[1].into())) {
            return Err(format!("duplicate test-binary exception: {line}"));
        }
    }
    Ok(Some(Configuration {
        workspaces,
        exceptions: declared,
    }))
}
