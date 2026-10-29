pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) fn collect_inc_rs_files(root: &Path, offenders: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    for entry in std::fs::read_dir(root).expect("crate directories should be readable") {
        let entry = entry.expect("directory entries should be readable");
        let path = entry.path();
        let file_type = entry.file_type().expect("file types should be readable");
        if file_type.is_dir() {
            collect_inc_rs_files(&path, offenders);
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".inc.rs"))
        {
            offenders.push(path);
        }
    }
}