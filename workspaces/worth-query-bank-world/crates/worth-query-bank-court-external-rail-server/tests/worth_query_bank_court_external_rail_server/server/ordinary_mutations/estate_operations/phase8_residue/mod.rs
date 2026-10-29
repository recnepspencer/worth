pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_residue::*;

#[test]
fn superseded_aftermath_authorities_are_removed() {
    // Monolith path — removed (directory absent).
    let monolith = concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-server"),
        "/../../../worth-query/crates/worth-query/src/domain_installation/operation_aftermath"
    );
    assert!(
        !std::path::Path::new(monolith).exists(),
        "monolith operation_aftermath must be removed, not privatized"
    );

    // Bank-local EstateAftermath enum — removed (only residue strings in denial scan).
    let bank_aftermath = include_str!(concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-server"),
        "/../bank-domain/src/estate/aftermath.rs"
    ));
    assert!(
        !bank_aftermath.contains("enum EstateAftermath"),
        "bank-local EstateAftermath enum must be removed"
    );
    assert!(
        bank_aftermath.contains("declared_aftermath_for"),
        "destination declared_aftermath_for must remain the sole bank declaration path"
    );

    // Generic Phase-8 mutation rollback door — removed from aftermath surface.
    let bank_server_src = concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-server"),
        "/src"
    );
    let mut rollback_hits = Vec::new();
    for entry in walkdir_rs(bank_server_src) {
        if !entry.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&entry) else {
            continue;
        };
        if text.contains("fn rollback") || text.contains("generic_rollback") {
            rollback_hits.push(entry);
        }
    }
    assert!(
        rollback_hits.is_empty(),
        "bank-server must not expose a Phase-8 generic rollback door: {rollback_hits:?}"
    );
}

#[test]
fn ordinary_paths_do_not_call_for_replay_retention() {
    let aftermath_root = concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-server"),
        "/../../../worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath"
    );
    let mut hits = Vec::new();
    for entry in walkdir_rs(aftermath_root) {
        if !entry.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&entry) else {
            continue;
        };
        if text.contains("_for_replay") || text.contains("for_replay") {
            hits.push(entry);
        }
    }
    assert!(
        hits.is_empty(),
        "ordinary Phase 8 aftermath paths must not name for_replay APIs: {hits:?}"
    );
}

#[test]
fn no_signal_decision_classifies_aftermath() {
    let aftermath_root = concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-server"),
        "/../../../worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath"
    );
    let mut hits = Vec::new();
    for entry in walkdir_rs(aftermath_root) {
        if !entry.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&entry) else {
            continue;
        };
        if text.contains("worth_signal") || text.contains("WorthSignal") {
            hits.push(entry);
        }
    }
    assert!(
        hits.is_empty(),
        "Signal must not classify aftermath: {hits:?}"
    );
}
