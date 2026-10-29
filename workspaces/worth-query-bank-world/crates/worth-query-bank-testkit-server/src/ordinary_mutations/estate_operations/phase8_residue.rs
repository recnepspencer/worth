//! R8.49 / R8.50 / R8.3 / R8.11 — supplemental mechanical residue.
//!
//! Source scans may prove absence or physical retirement, but never Rust facade
//! reachability. Accepted/provisional reachability is owned by Query trybuild
//! positive and negative twins.

pub fn walkdir_rs(root: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir_rs(path.to_str().unwrap_or("")));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path.display().to_string());
        }
    }
    out
}
