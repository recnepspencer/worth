#[allow(unused_imports)] use crate::physical_work_recovery_source_boundary::*;
use {std::path::Path, std::path::PathBuf};

#[test]
fn physical_work_recovery_has_no_raw_decoder_route() {
    let store_source = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store")).join("src");
    let production_sources = production_rust_sources(&store_source);

    assert!(!production_sources.is_empty());
    for source_path in production_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in ["decode_physical_work_obligation_v6", "decode_locator"] {
            assert!(
                !source.contains(forbidden),
                "Store production source {} reopened forbidden physical-work raw route {forbidden}",
                source_path.display()
            );
        }
    }
}
