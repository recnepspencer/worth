pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) fn production_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry.expect("Store source entry must be readable").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name != "tests") {
                    pending.push(path);
                }
                continue;
            }
            if is_production_rust_source(&path) {
                sources.push(path);
            }
        }
    }
    sources
}

pub(crate) fn is_production_rust_source(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.extension().is_some_and(|extension| extension == "rs")
        && file_name != "tests.rs"
        && !file_name.ends_with("_tests.rs")
}
