#[allow(unused_imports)]
use crate::no_inc_files::ordinary_context::*;
use {std::path::Path, std::path::PathBuf};

#[test]
fn worth_runtime_bridge_disallows_inc_rs_files() {
    let crate_root = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-runtime-bridge"
    ));
    let mut offenders = Vec::new();

    let product_sources = crate_root.join("src");
    let court_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    assert!(
        product_sources.is_dir(),
        "Bridge product sources must exist"
    );
    assert!(court_tests.is_dir(), "Bridge court tests must exist");
    collect_inc_rs_files(&product_sources, &mut offenders);
    collect_inc_rs_files(&court_tests, &mut offenders);

    assert!(
        offenders.is_empty(),
        "worth-runtime-bridge should not contain `.inc.rs` files.\nOffenders:\n{}",
        offenders
            .iter()
            .map(|path| path
                .strip_prefix(&crate_root)
                .unwrap_or(path)
                .display()
                .to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
