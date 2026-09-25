use super::super::crate_modules::{ModuleGraph, ModuleNode};
use super::{check_graphs, enforce_raw_geometry_denials};
use crate::config::{RawGeometryDenialConfig, RawGeometryEdgeConfig, RawGeometryEdgeKind};
use std::collections::BTreeMap;

fn graph(modules: &[(&[&str], &str, &str)]) -> ModuleGraph {
    ModuleGraph {
        modules: modules
            .iter()
            .map(|(path, source, text)| {
                (
                    path.iter().map(|segment| (*segment).to_owned()).collect(),
                    ModuleNode {
                        relative_source: (*source).to_owned(),
                        public_from_parent: false,
                        items: syn::parse_file(text).unwrap().items,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    }
}

fn edge(path: &str, items: &[&str]) -> RawGeometryEdgeConfig {
    RawGeometryEdgeConfig {
        path: path.to_owned(),
        items: items.iter().map(|item| (*item).to_owned()).collect(),
        kind: RawGeometryEdgeKind::Serialization,
        reason: "wire data".to_owned(),
    }
}

fn rule(edges: Vec<RawGeometryEdgeConfig>) -> RawGeometryDenialConfig {
    RawGeometryDenialConfig {
        crate_root: "crates/covered".to_owned(),
        guidance: "carry the sealed type".to_owned(),
        edges,
    }
}

fn subjects(source: &str, edges: Vec<RawGeometryEdgeConfig>) -> Vec<String> {
    check_graphs(&[graph(&[(&[], "src/lib.rs", source)])], &rule(edges))
        .into_iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.subject(), diagnostic.message()))
        .collect()
}

#[test]
fn every_written_coordinate_shape_is_rejected_outside_an_edge() {
    for source in [
        "fn f(point: [f32; 2]) {}",
        "fn f() -> [f64; 4] { todo!() }",
        "struct S { offset: [f64; 2] }",
        "enum E { At([f32; 2]) }",
        "fn f() { let range: (f32, f32) = todo!(); }",
        "fn f() { let _ = |point: [f64; 2]| point; }",
        "fn f() -> Box<[[f32; 4]]> { todo!() }",
        "type Point = [f32; 2];",
        "impl S { fn at(self) -> (f64, f32) { todo!() } }",
        "trait T { fn at(&self) -> [f32; 2]; }",
        "#[cfg(feature = \"certification-support\")] fn f(point: [f32; 2]) {}",
    ] {
        assert_eq!(subjects(source, Vec::new()).len(), 1, "accepted {source}");
    }
}

#[test]
fn other_shapes_and_code_compiled_out_of_production_are_not_raw_coordinates() {
    for source in [
        "fn f(color: [u8; 4], extent: [u32; 2], triple: [f32; 3], pair: (f32, u32)) {}",
        "fn f<const N: usize>(values: [f32; N]) {}",
        "#[cfg(test)] fn f(point: [f32; 2]) {}",
        "#[cfg(test)] mod tests { fn f(point: [f32; 2]) {} }",
        "struct S { #[cfg(test)] probe: [f32; 2] }",
        "impl S { #[cfg(all(test, not(feature = \"x\")))] fn f(point: [f32; 2]) {} }",
        "fn f() { let _ = \"[f32; 2]\"; }",
    ] {
        assert!(subjects(source, Vec::new()).is_empty(), "rejected {source}");
    }
}

#[test]
fn a_file_module_compiled_out_of_production_is_skipped() {
    let graph = graph(&[
        (&[], "src/lib.rs", "#[cfg(test)] mod probe;"),
        (&["probe"], "src/probe.rs", "fn f(point: [f32; 2]) {}"),
    ]);
    assert!(check_graphs(&[graph], &rule(Vec::new())).is_empty());
}

#[test]
fn an_edge_covers_exactly_its_source_and_items() {
    let source = "impl Bounds { fn admit(p: [f32; 2]) {} fn leak(p: [f32; 2]) {} }";
    let found = subjects(source, vec![edge("src/lib.rs", &["Bounds::admit"])]);
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("`Bounds::leak`"), "{found:?}");
    assert!(subjects(source, vec![edge("src/lib.rs", &["Bounds"])]).is_empty());
    assert!(subjects(source, vec![edge("src/lib.rs", &[])]).is_empty());
    assert!(subjects(source, vec![edge("src", &[])]).is_empty());
    // A path prefix is not a directory: `src/li` does not hold `src/lib.rs`.
    assert_eq!(subjects(source, vec![edge("src/li", &[])]).len(), 3);
    // Nor is an item prefix an item: `Bounds::adm` holds nothing.
    assert_eq!(
        subjects(source, vec![edge("src/lib.rs", &["Bounds::adm"])]).len(),
        3
    );
}

#[test]
fn an_inline_module_is_named_in_the_file_that_declares_it() {
    let graph = graph(&[
        (&[], "src/lib.rs", "mod wire { fn bits(v: [f32; 2]) {} }"),
        (&["wire"], "src/lib.rs", "fn bits(v: [f32; 2]) {}"),
    ]);
    assert!(check_graphs(
        std::slice::from_ref(&graph),
        &rule(vec![edge("src/lib.rs", &["wire::bits"])])
    )
    .is_empty());
    let found = check_graphs(&[graph], &rule(Vec::new()));
    assert_eq!(found.len(), 1, "an inline module is read once");
    assert!(found[0].message().contains("`wire::bits`"));
}

#[test]
fn every_production_target_is_read_and_a_shared_source_is_reported_once() {
    let wire = "fn f(p: [f32; 2]) {}";
    let library = graph(&[
        (&[], "src/lib.rs", r#"#[path = "wire.rs"] mod wire;"#),
        (&["wire"], "src/wire.rs", wire),
    ]);
    let binary = graph(&[
        (
            &[],
            "src/main.rs",
            r#"mod layout; #[path = "wire.rs"] mod wire;"#,
        ),
        (
            &["layout"],
            "src/layout.rs",
            "fn resolve() -> (f32, f32) { todo!() }",
        ),
        (&["wire"], "src/wire.rs", wire),
    ]);
    let found = check_graphs(&[library, binary], &rule(Vec::new()));
    let subjects: Vec<_> = found
        .iter()
        .map(|diagnostic| diagnostic.subject())
        .collect();
    assert_eq!(
        subjects,
        [
            "crates/covered/src/layout.rs:1",
            "crates/covered/src/wire.rs:1"
        ]
    );
}

#[test]
fn an_edge_that_holds_no_raw_form_is_stale() {
    let found = subjects("fn f() {}", vec![edge("src/lib.rs", &["f"])]);
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("crates/covered/src/lib.rs: declared serialization edge"));
}

#[test]
fn a_missing_covered_crate_fails_closed() {
    let root = std::env::temp_dir().join(format!("boundary-raw-geometry-{}", std::process::id()));
    let found = enforce_raw_geometry_denials(&root, &[rule(Vec::new())]);
    assert_eq!(found.len(), 1);
    assert!(found[0].message().contains("could not be read"));
}
