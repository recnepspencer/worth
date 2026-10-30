use super::*;
use crate::filesystem_media::tests::allocation_probe;

#[test]
fn qualified_component_open_bounds_cover_unicode_growth_and_missing_file() {
    if !allocation_probe::alone_in_its_process(
        module_path!(),
        "qualified_component_open_bounds_cover_unicode_growth_and_missing_file",
    ) {
        return;
    }
    assert!(qualified(), "the pinned native compiler must be qualified");
    let root = tempfile::tempdir().unwrap();
    let directory =
        cap_std::fs::Dir::open_ambient_dir(root.path(), cap_std::ambient_authority()).unwrap();
    // Small-buffer growth, Unicode uppercase expansion, leading-dot stems,
    // supplementary characters, and a near-maximum single Windows component.
    for name in [
        "a".to_owned(),
        "ßﬃ-界-🦀.wal".to_owned(),
        ".ßﬃ.wal".to_owned(),
        format!("{}.wal", "界".repeat(246)),
    ] {
        std::fs::write(root.path().join(&name), b"payload").unwrap();
        let bound = open_bytes(&name).unwrap();
        // cap's device-name check is disposed before normal opening begins.
        // Census that exact prefix's uppercase operation separately: cumulative
        // requests include both phases, whereas open_bytes bounds their peak.
        // All fixture prefixes are either the whole name or its .wal stem.
        let stem = name.strip_suffix(".wal").unwrap_or(&name);
        let uppercase = allocation_probe::allocated_bytes_during(|| {
            drop(stem.trim_end().to_uppercase());
        });
        assert!(uppercase as u64 <= bound);
        let mut opened = None;
        let requested = allocation_probe::allocated_bytes_during(|| {
            opened = Some(directory.open(&name).unwrap());
        });
        assert!(
            requested as u64 <= bound + uppercase as u64,
            "{name}: {requested} exceeds the peak envelope {bound} plus disposed uppercase {uppercase}"
        );
        assert_eq!(opened.unwrap().metadata().unwrap().len(), 7);
        std::fs::remove_file(root.path().join(&name)).unwrap();
        let mut failed = None;
        let missing = allocation_probe::allocated_bytes_during(|| {
            failed = Some(directory.open(&name));
        });
        assert_eq!(
            failed.unwrap().unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert!(
            missing as u64 <= bound + uppercase as u64,
            "missing {name}: {missing} exceeds {bound} plus disposed uppercase {uppercase}"
        );
        // Only observations, never replacement limits or self-derived oracles.
        eprintln!(
            "component {} UTF-8 bytes: open {requested}, missing {missing}, disposed uppercase {uppercase}, peak bound {bound}",
            name.len()
        );
    }
}
