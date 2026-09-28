use std::fs;
use std::path::{Path, PathBuf};

/// Replace every governed snapshot together, or leave all of them untouched.
pub(super) fn commit_snapshots(files: &[(PathBuf, String)]) -> Result<Vec<PathBuf>, String> {
    let paths = files
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let rendered = files
        .iter()
        .map(|(_, text)| text.clone())
        .collect::<Vec<_>>();
    replace_all(&paths, &rendered, |from, to| fs::rename(from, to))?;
    Ok(paths)
}

fn replace_all<F>(paths: &[PathBuf], rendered: &[String], mut rename: F) -> Result<(), String>
where
    F: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    let nonce = format!("{}.snapshot-update", std::process::id());
    let stages = paths
        .iter()
        .map(|path| path.with_extension(format!("toml.{nonce}.stage")))
        .collect::<Vec<_>>();
    let backups = paths
        .iter()
        .map(|path| path.with_extension(format!("toml.{nonce}.backup")))
        .collect::<Vec<_>>();
    for ((path, stage), text) in paths.iter().zip(&stages).zip(rendered) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(stage, text).map_err(|e| format!("stage {}: {e}", stage.display()))?;
    }
    let existed = paths.iter().map(|path| path.exists()).collect::<Vec<_>>();
    for index in 0..paths.len() {
        if existed[index] {
            if let Err(error) = rename(&paths[index], &backups[index]) {
                rollback_backups(paths, &backups, &existed, index, &mut rename);
                cleanup(&stages);
                return Err(format!("backup {}: {error}", paths[index].display()));
            }
        }
    }
    for index in 0..paths.len() {
        if let Err(error) = rename(&stages[index], &paths[index]) {
            for committed_path in paths.iter().take(index) {
                let _ = fs::remove_file(committed_path);
            }
            rollback_backups(paths, &backups, &existed, paths.len(), &mut rename);
            cleanup(&stages);
            return Err(format!("replace {}: {error}", paths[index].display()));
        }
    }
    cleanup(&backups);
    Ok(())
}

fn rollback_backups<F>(
    paths: &[PathBuf],
    backups: &[PathBuf],
    existed: &[bool],
    count: usize,
    rename: &mut F,
) where
    F: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    for index in 0..count {
        if existed[index] && backups[index].exists() {
            let _ = rename(&backups[index], &paths[index]);
        }
    }
}

fn cleanup(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn late_replacement_failure_restores_the_original_set() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("boundary-snapshot-commit-{id}"));
        let paths = [
            root.join("crate-dag.toml"),
            root.join("facades.toml"),
            root.join("facade-doc-debt.toml"),
        ];
        fs::create_dir_all(&root).unwrap();
        fs::write(&paths[0], "old dag").unwrap();
        fs::write(&paths[1], "old facades").unwrap();
        let mut replacements = 0;
        let result = replace_all(
            &paths,
            &["new dag".into(), "new facades".into(), "new debt".into()],
            |from, to| {
                if from.extension().and_then(|value| value.to_str()) == Some("stage") {
                    replacements += 1;
                    if replacements == 3 {
                        return Err(std::io::Error::other("injected third replacement failure"));
                    }
                }
                fs::rename(from, to)
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&paths[0]).unwrap(), "old dag");
        assert_eq!(fs::read_to_string(&paths[1]).unwrap(), "old facades");
        assert!(!paths[2].exists());
    }
}
