//! Acquire each requested physical file once, independently across workers.
use super::{current, ReadFailure};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) fn prefetch(paths: impl IntoIterator<Item = PathBuf>) {
    let Some(documents) = current() else {
        return;
    };
    let width = documents.borrow().width;
    let paths: Vec<_> = paths
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let unresolved: Vec<_> = paths
        .iter()
        .filter(|path| !documents.borrow().aliases.contains_key(*path))
        .collect();
    let resolved = crate::source_workers::map(&unresolved, width, |path| {
        (
            (*path).clone(),
            std::fs::canonicalize(path).unwrap_or_else(|_| (*path).clone()),
        )
    });
    {
        let mut documents = documents.borrow_mut();
        documents.aliases.extend(resolved);
    }
    let missing: Vec<_> = {
        let documents = documents.borrow();
        paths
            .iter()
            .filter_map(|path| {
                let key = &documents.aliases[path];
                (!documents.bytes.contains_key(key)).then(|| key.clone())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    let acquired = crate::scan_profile::measure("parallel file acquisition", || {
        crate::source_workers::map(&missing, width, |path| {
            crate::scan_profile::measure("physical file read", || std::fs::read(path))
                .map_err(ReadFailure::capture)
        })
    });
    let mut documents = documents.borrow_mut();
    for (path, result) in missing.into_iter().zip(acquired) {
        if path.extension().is_some_and(|value| value == "rs") {
            documents.counts.reads += 1;
        }
        documents.bytes.insert(path, result.map(Rc::<[u8]>::from));
    }
}
