mod module_graph;

use std::collections::BTreeSet;
use std::path::Path;

use module_graph::{
    closure_digest, module_closure, resolved_pure_closure, validate_oracle_imports,
    visible_reexports, PureGraphPolicy,
};

const ORACLE_ROOT: &str =
    "src/tests/domains/fintech/certification/invalidation/locality_expectation.rs";
const WORLD_FACADE: &str = "src/tests/domains/fintech/world/mod.rs";
const PURE_WORLD_OWNERS: &[&str] = &[
    "locality_definition",
    "locality_scale",
    "market_inputs",
    "positions",
];
const ORACLE_CLOSURE_DIGEST: &str =
    "609c208becd1af0d08db662890e7922081d1d62a97bdeda44869f2457740f052";
const PURE_WORLD_CLOSURE_DIGEST: &str =
    "655b53ad30ce7fdd902848e766041deb9771a40e04b85abc3bdcd511507f02e9";

#[test]
fn complete_oracle_dependency_graph_excludes_runtime_authority() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!manifest.join("build.rs").exists());
    let oracle_files = module_closure(&manifest.join(ORACLE_ROOT), None)
        .expect("oracle module closure must resolve");
    assert_eq!(oracle_files.len(), 11);
    assert_eq!(
        closure_digest(manifest, &oracle_files),
        ORACLE_CLOSURE_DIGEST,
        "oracle closure identity changed"
    );

    let facade_source = std::fs::read_to_string(manifest.join(WORLD_FACADE))
        .expect("world facade must be readable");
    let reexports = visible_reexports(&facade_source).expect("world facade must be unambiguous");
    let oracle_owners = validate_oracle_imports(manifest, &oracle_files, &reexports)
        .expect("oracle imports must resolve only to pure financial meaning");
    assert_eq!(
        oracle_owners,
        ["locality_definition", "locality_scale", "positions"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );

    let pure_closure = resolved_pure_closure(
        manifest,
        WORLD_FACADE,
        &oracle_owners,
        PureGraphPolicy {
            allowed: PURE_WORLD_OWNERS,
            reexports: &reexports,
        },
    )
    .expect("pure financial dependency closure must resolve");
    assert_eq!(
        pure_closure.owners,
        PURE_WORLD_OWNERS
            .iter()
            .map(|owner| (*owner).to_owned())
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        closure_digest(manifest, &pure_closure.files),
        PURE_WORLD_CLOSURE_DIGEST,
        "pure world dependency closure identity changed"
    );
}

#[test]
fn oracle_dependency_guard_rejects_direct_and_reexported_runtime_imports() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join(ORACLE_ROOT);
    let source = std::fs::read_to_string(&root).expect("oracle root must be readable");
    let direct = format!("{source}\nuse crate::logic::invalidation::mark_dirty;");
    let closure = module_closure(&root, Some(&direct)).expect("mutated root must parse");
    let facade = std::fs::read_to_string(manifest.join(WORLD_FACADE))
        .expect("world facade must be readable");
    let reexports = visible_reexports(&facade).expect("world facade must resolve");
    assert!(validate_oracle_imports(manifest, &closure, &reexports).is_err());

    let aliased =
        format!("{facade}\npub(super) use crate::facade::mark_dirty as oracle_mark_dirty;");
    let aliased = visible_reexports(&aliased).expect("alias must parse");
    assert_eq!(
        aliased
            .get("oracle_mark_dirty")
            .and_then(module_graph::WorldReexport::unconditional_origin),
        Some("crate::facade::mark_dirty")
    );

    let chained =
        format!("{facade}\nmod runtime_bridge;\npub(super) use runtime_bridge::oracle_mark_dirty;");
    let chained = visible_reexports(&chained).expect("chained alias must parse");
    assert_eq!(
        chained
            .get("oracle_mark_dirty")
            .and_then(module_graph::WorldReexport::unconditional_origin),
        Some("runtime_bridge::oracle_mark_dirty")
    );
}

#[test]
fn oracle_dependency_guard_rejects_new_or_hidden_modules() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join(ORACLE_ROOT);
    let source = std::fs::read_to_string(&root).expect("oracle root must be readable");
    for bridge in [
        "mod runtime_bridge;",
        "#[cfg(not(test))] mod runtime_bridge;",
        "#[path = \"elsewhere.rs\"] mod runtime_bridge;",
        "mod runtime_bridge { use crate::facade::SignalGraph; }",
    ] {
        let mutated = format!("{source}\n{bridge}");
        assert!(module_closure(&root, Some(&mutated)).is_err());
    }
}

#[test]
fn pure_dependency_guard_rejects_relative_glob_and_macro_escapes() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = std::fs::read_to_string(manifest.join(WORLD_FACADE))
        .expect("world facade must be readable");
    let reexports = visible_reexports(&facade).expect("world facade must resolve");
    let positions = manifest.join("src/tests/domains/fintech/world/positions.rs");
    let source = std::fs::read_to_string(&positions).expect("positions must be readable");
    for escape in [
        "use super::compiler::CompiledFinancialWorld;",
        "use super::compiler::*;",
        "include!(\"runtime_bridge.rs\");",
    ] {
        let mutated = format!("{source}\n{escape}");
        assert!(module_graph::validate_pure_source_mutation(
            manifest,
            &positions,
            &mutated,
            PureGraphPolicy {
                allowed: PURE_WORLD_OWNERS,
                reexports: &reexports,
            },
        )
        .is_err());
    }
}
