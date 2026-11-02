//! One Git source inventory for repository and consumer binding checks.
use super::command;
use crate::config::{Files, CONFIG};
use std::{collections::BTreeSet, fs, path::Path, thread};

pub(super) fn read(root: &Path) -> Result<(BTreeSet<String>, Files), String> {
    let mut paths = BTreeSet::new();
    for args in [
        vec!["ls-files", "-z"],
        vec!["ls-files", "-z", "-o", "--exclude-standard"],
    ] {
        let output = command(root, "git", &args)?;
        for path in output.split(|b| *b == 0).filter(|p| !p.is_empty()) {
            let path = String::from_utf8(path.to_vec()).map_err(|e| e.to_string())?;
            paths.insert(path);
        }
    }
    for path in command(root, "git", &["ls-files", "--deleted", "-z"])?
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
    {
        paths.remove(std::str::from_utf8(path).map_err(|e| e.to_string())?);
    }
    let needed: Vec<_> = paths
        .iter()
        .filter(|p| {
            p.ends_with(".rs")
                || p.ends_with(".md")
                || p.ends_with(".toml")
                || p.starts_with(CONFIG)
        })
        .collect();
    let width = thread::available_parallelism().map_or(1, usize::from);
    let files = thread::scope(|s| {
        let handles: Vec<_> = needed
            .chunks(needed.len().div_ceil(width).max(1))
            .map(|chunk| {
                s.spawn(move || {
                    chunk
                        .iter()
                        .map(|p| {
                            fs::read_to_string(root.join(p))
                                .map(|text| ((***p).to_owned(), text.replace("\r\n", "\n")))
                                .map_err(|e| format!("{p}: {e}"))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("file reader panicked"))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.into_iter().flatten().collect::<Files>())
    })?;
    Ok((paths, files))
}
