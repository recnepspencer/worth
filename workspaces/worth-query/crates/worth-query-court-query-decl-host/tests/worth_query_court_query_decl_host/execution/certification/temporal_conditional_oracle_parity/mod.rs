#[test]
fn temporal_conditional_identity_has_no_private_digest_engine() {
    let owner = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-query-certification"
    ))
    .join("../worth-query-execution/src/domain_computation/primary_graph/conditional_operation");
    let mut pending = vec![owner];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                assert!(!source.contains("use sha2"), "{}", path.display());
                assert!(!source.contains("Sha256::"), "{}", path.display());
            }
        }
    }
}
