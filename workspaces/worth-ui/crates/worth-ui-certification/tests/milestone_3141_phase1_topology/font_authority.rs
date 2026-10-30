use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syn::visit::Visit;

const TEXT_DEPENDENCIES: &[&str] = &[
    "harfrust",
    "icu_segmenter",
    "kurbo",
    "linesweeper",
    "png",
    "read-fonts",
    "sha2",
    "skrifa",
    "swash",
    "unicode-bidi",
    "unicode-segmentation",
    "worth-ui-host-contract",
];

#[test]
fn headless_measurement_and_accessibility_consume_only_qualified_records() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory");
    let headless = crates.join("worth-ui-host-headless/src/headless_transcript");
    let measurement = std::fs::read_to_string(headless.join("text_measurement.rs"))
        .expect("measurement consumer source is readable");
    let accessibility = std::fs::read_to_string(headless.join("text_accessibility.rs"))
        .expect("accessibility consumer source is readable");

    validate_measurement_consumer(&measurement).expect("measurement consumes qualified records");
    validate_accessibility_consumer(&accessibility)
        .expect("accessibility consumes qualified records");
    assert!(validate_accessibility_consumer(
        "impl Geometry { fn lines(&self) -> &[Line] { &[] } }"
    )
    .is_err());
    assert!(validate_accessibility_consumer(
        "impl Geometry { fn lines(&self) -> &[Line] { self.mechanic.reshape() } }"
    )
    .is_err());
    assert!(validate_measurement_consumer(
        "impl Mechanic { fn qualified_measurement(&self) { alternate_shape(self); } }"
    )
    .is_err());
}

#[test]
fn qualified_text_has_no_ambient_system_font_authority() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory")
        .join("worth-ui-text");
    let manifest = std::fs::read_to_string(crate_root.join("Cargo.toml"))
        .expect("worth-ui-text manifest is readable");
    validate_manifest(&manifest).expect("text dependencies are exact and platform neutral");

    for source in rust_sources(&crate_root.join("src")) {
        let text = std::fs::read_to_string(&source).expect("text source is readable");
        validate_runtime_source(&text)
            .unwrap_or_else(|error| panic!("{}: {error}", source.display()));
    }

    let mut widened: toml::Value = toml::from_str(&manifest).unwrap();
    widened["dependencies"]
        .as_table_mut()
        .unwrap()
        .insert("fontdb".to_owned(), toml::Value::Table(toml::Table::new()));
    assert!(validate_manifest(&widened.to_string()).is_err());
    assert!(
        validate_runtime_source("fn load() { std::fs::read(\"C:/Windows/Fonts/a.ttf\"); }")
            .is_err()
    );
    assert!(validate_runtime_source("extern \"system\" { fn system_font(); }").is_err());
}

#[test]
fn qualified_color_sources_have_one_shared_font_collection_owner() {
    let text = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory")
        .join("worth-ui-text/src/font_collection");
    assert!(text.join("color_glyph.rs").is_file());
    assert!(text.join("color_glyph/bitmap_selection.rs").is_file());
    assert!(text.join("color_glyph/path.rs").is_file());
    for displaced in [
        "application_pack/color_tables.rs",
        "application_pack/color_tables",
        "ink_bounds/bitmap_selection.rs",
        "ink_bounds/color_path.rs",
    ] {
        assert!(
            !text.join(displaced).exists(),
            "shared color source remains under consumer {displaced}"
        );
    }
    for consumer in [
        "face.rs",
        "application_pack/metadata.rs",
        "ink_bounds/bitmap.rs",
        "ink_bounds/color.rs",
        "../raster/color/bitmap.rs",
        "../raster/color/colr.rs",
    ] {
        let source = std::fs::read_to_string(text.join(consumer))
            .unwrap_or_else(|error| panic!("{consumer}: {error}"));
        assert!(
            source.contains("color_glyph"),
            "{consumer} bypasses the shared qualified color source owner"
        );
    }
}

#[test]
fn runtime_alone_owns_qualified_layouts_while_hosts_receive_borrowed_views() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory");
    for name in [
        "worth-ui-host-contract",
        "worth-ui-host-headless",
        "worth-ui-host-native",
    ] {
        for source in rust_sources(&crates.join(name).join("src")) {
            let text = std::fs::read_to_string(&source).expect("host source is readable");
            validate_host_layout_source(&text)
                .unwrap_or_else(|error| panic!("{}: {error}", source.display()));
        }
    }
    let runtime_owner = std::fs::read_to_string(
        crates.join("worth-ui-runtime/src/mounting/projection/semantic_text/qualified.rs"),
    )
    .expect("runtime qualified-text owner is readable");
    assert!(runtime_owner.contains("layout: Arc<worth_ui_text::UiQualifiedTextLayout>"));
    let contract =
        std::fs::read_to_string(crates.join("worth-ui-host-contract/src/qualified_text/view.rs"))
            .expect("borrowed layout contract is readable");
    assert!(contract.contains("UiQualifiedTextLayoutView<'layout>"));
    assert!(validate_host_layout_source(
        "struct Host { layout: std::sync::Arc<worth_ui_text::UiQualifiedTextLayout> }"
    )
    .is_err());
    assert!(validate_host_layout_source("struct Host(UiQualifiedTextLayoutArtifact);").is_err());
}

fn validate_host_layout_source(source: &str) -> Result<(), String> {
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.contains("UiQualifiedTextLayoutArtifact")
        || compact.contains("Arc<worth_ui_text::UiQualifiedTextLayout")
        || compact.contains("Arc<UiQualifiedTextLayout")
    {
        Err("host retains framework-owned qualified layout authority".to_owned())
    } else {
        Ok(())
    }
}

fn validate_accessibility_consumer(source: &str) -> Result<(), String> {
    for method in ["layout_identity", "lines", "visual_runs", "carets"] {
        let calls = calls_in(source, method)?;
        if calls.method_calls != [method] || !calls.function_calls.is_empty() {
            return Err(format!(
                "accessibility {method} does not directly borrow the qualified record: {calls:?}"
            ));
        }
    }
    Ok(())
}

fn validate_measurement_consumer(source: &str) -> Result<(), String> {
    let calls = calls_in(source, "qualified_measurement")?;
    let observed = calls
        .method_calls
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = [
        "lines",
        "iter",
        "map",
        "max",
        "unwrap_or",
        "layout_identity",
        "first",
        "bounds",
        "bottom_millipoints",
        "baseline_millipoints",
        "logical_bounds",
        "ink_bounds",
        "profile_generation",
        "font_collection_generation",
        "text_scale_generation",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if observed != expected || calls.function_calls != ["content_width"] {
        return Err(format!(
            "measurement introduced an independent layout pass: {calls:?}"
        ));
    }
    Ok(())
}

#[derive(Debug, Default)]
struct CallInventory {
    method_calls: Vec<String>,
    function_calls: Vec<String>,
}

fn calls_in(source: &str, function: &str) -> Result<CallInventory, String> {
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let item = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| item.items.iter())
        .filter_map(|item| match item {
            syn::ImplItem::Fn(item) if item.sig.ident == function => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    if item.len() != 1 {
        return Err(format!(
            "expected one {function} consumer, found {}",
            item.len()
        ));
    }
    let mut inventory = CallInventory::default();
    inventory.visit_block(&item[0].block);
    Ok(inventory)
}

impl<'ast> Visit<'ast> for CallInventory {
    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        self.method_calls.push(expression.method.to_string());
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*expression.func
            && let Some(segment) = path.path.segments.last()
        {
            self.function_calls.push(segment.ident.to_string());
        }
        syn::visit::visit_expr_call(self, expression);
    }
}

fn validate_manifest(source: &str) -> Result<(), String> {
    let manifest: toml::Value = toml::from_str(source).map_err(|error| error.to_string())?;
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "text dependencies are absent".to_owned())?;
    let observed = dependencies
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = TEXT_DEPENDENCIES.iter().copied().collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(format!(
            "text dependency authority drifted: {observed:?} != {expected:?}"
        ));
    }
    for dependency in TEXT_DEPENDENCIES {
        let posture = dependencies[*dependency]
            .as_table()
            .ok_or_else(|| format!("{dependency} must use the workspace-qualified pin"))?;
        if posture.len() != 1
            || posture.get("workspace").and_then(toml::Value::as_bool) != Some(true)
        {
            return Err(format!("{dependency} dependency posture drifted"));
        }
    }
    Ok(())
}

fn validate_runtime_source(source: &str) -> Result<(), String> {
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut visitor = AmbientFontAuthority::default();
    visitor.visit_file(&syntax);
    if visitor.denials.is_empty() {
        Ok(())
    } else {
        Err(visitor.denials.join("; "))
    }
}

#[derive(Default)]
struct AmbientFontAuthority {
    denials: Vec<String>,
}

impl<'ast> Visit<'ast> for AmbientFontAuthority {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let identity = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if [
            "std::fs",
            "std::process",
            "windows",
            "windows_sys",
            "libloading",
            "fontdb",
            "font_kit",
            "dwrote",
        ]
        .iter()
        .any(|prefix| identity == *prefix || identity.starts_with(&format!("{prefix}::")))
        {
            self.denials
                .push(format!("ambient font authority path {identity}"));
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        self.denials
            .push("foreign system API declaration".to_owned());
        syn::visit::visit_item_foreign_mod(self, item);
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast syn::ExprUnsafe) {
        self.denials.push("unsafe ambient font escape".to_owned());
        syn::visit::visit_expr_unsafe(self, expression);
    }
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("text source directory is readable") {
            let path = entry.expect("text source entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs")
                && !path.file_stem().is_some_and(|stem| {
                    let stem = stem.to_string_lossy();
                    stem == "tests" || stem.ends_with("_tests")
                })
            {
                sources.push(path);
            }
        }
    }
    sources.sort();
    sources
}
