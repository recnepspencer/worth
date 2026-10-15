//! The repository's skill source and its confined generated Grok copy zone.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOTICE: &str = "Generated from repository skills/ by boundary-check --write-guidance.\nEdit skills/ only. Installed user skills are outside this generated zone.\n";
const MARKER: &str = "GENERATED.md";
pub(crate) struct Plan {
    pub writes: BTreeMap<PathBuf, Vec<u8>>,
    pub removes: Vec<PathBuf>,
}

pub(crate) fn planned(root: &Path) -> Result<Plan, String> {
    let source = root.join("skills");
    let destination = root.join(".grok/skills");
    no_link(&source)?;
    no_link(&root.join(".grok"))?;
    no_link(&destination)?;
    let authoritative = files(&source)?;
    if authoritative.is_empty() {
        return Err("skills/ has no authoritative files".into());
    }
    if authoritative.contains_key(Path::new(MARKER)) {
        return Err("reserved generated provenance path".into());
    }
    let actual = files(&destination)?;
    let previous = match actual.get(Path::new(MARKER)) {
        Some(bytes) => provenance(bytes)?,
        None => Default::default(),
    };
    let mut notice = NOTICE.to_owned();
    for path in authoritative.keys() {
        let name = path
            .to_str()
            .ok_or("non-UTF8 skill path")?
            .replace('\\', "/");
        if name.contains(['\n', '\r']) {
            return Err("unsupported skill path".into());
        }
        notice.push_str(&format!("- {name}\n"));
    }
    let mut expected = authoritative;
    expected.insert(MARKER.into(), notice.into_bytes());
    let mut removes = Vec::new();
    for path in actual.keys().filter(|path| !expected.contains_key(*path)) {
        if !previous.contains(path) {
            return Err(format!("unowned derived extra: {}", path.display()));
        }
        removes.push(destination.join(path));
    }
    let writes = expected
        .into_iter()
        .filter(|(path, bytes)| actual.get(path) != Some(bytes))
        .map(|(path, bytes)| (destination.join(path), bytes))
        .collect();
    Ok(Plan { writes, removes })
}

fn provenance(bytes: &[u8]) -> Result<std::collections::BTreeSet<PathBuf>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let normalized = text.replace("\r\n", "\n");
    let rows = normalized
        .strip_prefix(NOTICE)
        .ok_or("unknown generated provenance")?;
    let mut paths = std::collections::BTreeSet::new();
    for row in rows.lines() {
        let path = row
            .strip_prefix("- ")
            .ok_or("invalid generated path inventory")?;
        if (path.is_empty()
            || path.contains('\\')
            || !Path::new(path)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_))))
            || path == MARKER
            || !paths.insert(PathBuf::from(path))
        {
            return Err("invalid/duplicate generated path inventory".into());
        }
    }
    Ok(paths)
}

pub(crate) fn check(root: &Path) -> Result<(), String> {
    let plan = planned(root)?;
    if plan.writes.is_empty() && plan.removes.is_empty() {
        return Ok(());
    }
    Err(format!(
        "derived .grok/skills drift: {} missing/changed, {} stale; run --write-guidance",
        plan.writes.len(),
        plan.removes.len()
    ))
}

fn files(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    let mut result = BTreeMap::new();
    if !root.exists() {
        return Ok(result);
    }
    visit(root, root, &mut result)?;
    Ok(result)
}
fn visit(
    root: &Path,
    directory: &Path,
    result: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), String> {
    no_link(directory)?;
    for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        no_link(&path)?;
        if path.is_dir() {
            visit(root, &path, result)?;
        } else if path.is_file() {
            result.insert(
                path.strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_owned(),
                std::fs::read(&path).map_err(|e| e.to_string())?,
            );
        } else {
            return Err("unsupported skill source file kind".into());
        }
    }
    Ok(())
}
fn no_link(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.file_type().is_symlink() {
        return Err(format!("skill path is a link: {}", path.display()));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err("skill reparse points are unsupported".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_copy_drift_and_owned_stale_files_preserve_authority() {
        let root = worth_harness::tooling::fixture_path::unique_path("derived-skills");
        std::fs::create_dir_all(root.join("skills/example/references")).unwrap();
        std::fs::write(
            root.join("skills/example/references/doc.md"),
            b"authoritative",
        )
        .unwrap();
        std::fs::write(root.join("skills/extra.md"), b"formerly generated").unwrap();
        assert!(check(&root).is_err());
        let plan = planned(&root).unwrap();
        for (path, bytes) in plan.writes {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        assert!(check(&root).is_ok());
        std::fs::remove_file(root.join("skills/extra.md")).unwrap();
        std::fs::write(
            root.join(".grok/skills/example/references/doc.md"),
            b"edited",
        )
        .unwrap();
        let plan = planned(&root).unwrap();
        assert_eq!(plan.removes, [root.join(".grok/skills/extra.md")]);
        assert_eq!(plan.writes.len(), 2);
        std::fs::write(root.join(".grok/skills/local.md"), b"user").unwrap();
        assert!(planned(&root).is_err());
        assert_eq!(
            std::fs::read(root.join(".grok/skills/local.md")).unwrap(),
            b"user"
        );
        assert_eq!(
            std::fs::read(root.join("skills/example/references/doc.md")).unwrap(),
            b"authoritative"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unowned_extras_are_not_deleted() {
        let root = worth_harness::tooling::fixture_path::unique_path("unowned-skills");
        std::fs::create_dir_all(root.join("skills/example")).unwrap();
        std::fs::create_dir_all(root.join(".grok/skills")).unwrap();
        std::fs::write(root.join("skills/example/SKILL.md"), b"source").unwrap();
        std::fs::write(root.join(".grok/skills/extra.md"), b"user").unwrap();
        assert!(planned(&root).is_err());
        assert!(root.join(".grok/skills/extra.md").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn linked_destination_is_refused_without_touching_its_target() {
        let root = worth_harness::tooling::fixture_path::unique_path("linked-skills");
        std::fs::create_dir_all(root.join("skills/example")).unwrap();
        std::fs::write(root.join("skills/example/SKILL.md"), b"authoritative").unwrap();
        std::fs::create_dir_all(root.join(".grok")).unwrap();
        std::fs::create_dir_all(root.join("outside")).unwrap();
        std::fs::write(root.join("outside/user.md"), b"untouched").unwrap();
        std::os::unix::fs::symlink(root.join("outside"), root.join(".grok/skills")).unwrap();
        assert!(planned(&root).is_err());
        assert_eq!(
            std::fs::read(root.join("outside/user.md")).unwrap(),
            b"untouched"
        );
        std::fs::remove_file(root.join(".grok/skills")).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
