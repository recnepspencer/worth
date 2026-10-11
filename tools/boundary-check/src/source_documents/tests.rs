use super::{Counts, Session};
use crate::source_rules::{parse_crate_modules_where, GovernedCrate};
use std::path::Path;

#[test]
fn three_source_files_are_read_and_parsed_once_across_rule_visits() {
    let crate_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/source_acquisition");
    let governed = GovernedCrate {
        package: "source-acquisition-fixture".into(),
        relative_crate_root: "source-acquisition-fixture".into(),
        crate_root,
    };
    let session = Session::new(1);
    super::prefetch(
        ["lib.rs", "alpha.rs", "beta.rs"].map(|name| governed.crate_root.join("src").join(name)),
    );
    for _ in 0..2 {
        let graph = parse_crate_modules_where(&governed, &|_| true).unwrap();
        assert_eq!(graph.modules.len(), 3);
    }
    assert_eq!(
        session.counts(),
        Counts {
            reads: 3,
            parses: 3
        }
    );
}
