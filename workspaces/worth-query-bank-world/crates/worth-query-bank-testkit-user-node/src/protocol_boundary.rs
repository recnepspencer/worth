pub fn rust_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = vec![root.with_extension("rs")];
    if root.is_dir() {
        files.extend(
            std::fs::read_dir(root)
                .expect("protocol directory")
                .map(|entry| entry.expect("protocol entry").path())
                .filter(|path| path.extension().is_some_and(|extension| extension == "rs")),
        );
    }
    files
}
