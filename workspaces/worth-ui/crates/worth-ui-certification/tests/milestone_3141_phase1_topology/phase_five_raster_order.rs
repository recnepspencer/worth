use std::path::{Path, PathBuf};

#[test]
fn runtime_raster_calls_are_confined_to_planned_misses_or_cold_reconstruction() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory")
        .join("worth-ui-runtime/src");
    for source in production_rust_sources(&root) {
        let text = std::fs::read_to_string(&source).expect("runtime text source is readable");
        if source.ends_with("native_platform/text_presentation/rasterization.rs") {
            assert_planned_callback_owns_raster_calls(&text, &source);
        } else if source.ends_with("native_platform/text_presentation/transaction.rs") {
            assert_transaction_crosses_native_planning(&text, &source);
        } else {
            assert_preplan_raster_calls_absent(&text, &source);
        }
    }

    let discarded_call_mutant = "let _ = worth_ui_text::rasterize_alpha_outline(layout, demand);";
    assert!(contains_text_raster_call(discarded_call_mutant));
    assert!(!is_test_source(Path::new(
        "worth-ui-runtime/src/mounting/early_raster.rs"
    )));
}

fn assert_planned_callback_owns_raster_calls(source: &str, path: &Path) {
    let callback = source
        .find("impl UiGlyphRasterMissRasterizer for UiNativeTextMissRasterizer")
        .expect("runtime transaction owns the selected-miss callback");
    let (reconstruction_lane, planned_miss_lane) = source.split_at(callback);
    let reconstruction = reconstruction_lane
        .find("pub(crate) fn reconstruct_cache")
        .expect("runtime raster owner exposes the explicit cold-reconstruction lane");
    assert!(
        !contains_text_raster_call(&reconstruction_lane[..reconstruction]),
        "{} can rasterize before either admitted lane",
        path.display()
    );
    assert_eq!(
        text_raster_call_count(&reconstruction_lane[reconstruction..]),
        2,
        "{} cold reconstruction must rasterize only alpha and color selections",
        path.display()
    );
    assert_eq!(
        text_raster_call_count(planned_miss_lane),
        2,
        "{} planned-miss callback must rasterize only alpha and color selections",
        path.display()
    );
}

fn assert_transaction_crosses_native_planning(source: &str, path: &Path) {
    assert!(
        source.contains("UiMountedTextRasterWork::from_text_mechanics(")
            && source.contains("&callback,"),
        "{} bypasses the mounted-work planner or its selected-miss callback",
        path.display()
    );
    assert_preplan_raster_calls_absent(source, path);
}

fn assert_preplan_raster_calls_absent(source: &str, path: &Path) {
    assert!(
        !contains_text_raster_call(source),
        "{} can rasterize before native atlas planning admits misses",
        path.display()
    );
}

fn contains_text_raster_call(source: &str) -> bool {
    text_raster_call_count(source) != 0
}

fn text_raster_call_count(source: &str) -> usize {
    ["rasterize_alpha_outline", "rasterize_intrinsic_color"]
        .iter()
        .map(|name| source.matches(name).count())
        .sum()
}

fn production_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("runtime text-presentation owner exists") {
            let path = entry.expect("runtime text source entry is readable").path();
            if path.is_dir() && !is_test_source(&path) {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs")
                && !is_test_source(&path)
            {
                sources.push(path);
            }
        }
    }
    sources
}

fn is_test_source(path: &Path) -> bool {
    path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy();
        component == "tests"
            || component == "tests.rs"
            || component.starts_with("test_")
            || component.ends_with("_tests")
            || component.ends_with("_tests.rs")
            || component.ends_with("_test_support.rs")
            || component.ends_with("_test_world")
    })
}
