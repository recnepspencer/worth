use super::super::crate_modules::{parse_crate_modules, GovernedCrate};
use super::{check_source, enforce};

const NEW_CONSUMER: &str = "workspaces/worth-store/crates/worth-store/src/new_owner/new_reader.rs";

#[test]
fn direct_aliased_and_macro_raw_entries_are_rejected() {
    for source in [
        "fn read() { BootstrapCatalog::decode(bytes); }",
        "use worth_store_physical_format::BootstrapCatalog as Catalog; fn read() { Catalog::decode(bytes); }",
        "type Catalog = worth_store_physical_format::BootstrapCatalog; fn read() { Catalog::decode(bytes); }",
        "use worth_store_physical_format::decode_extent_chunk as parse;",
        "macro_rules! reader { () => { BootstrapCatalog::decode(bytes) }; }",
        "fn read() { PhysicalRootRoutingBlock::decode_bounded(bytes, limits); }",
        "fn read() { CheckpointStreamDecoder::begin(bytes); }",
        "fn read() { durable_artifact_checksum(bytes); }",
        "fn read() { headers.decode_page_header(bytes); }",
        "fn read() { CheckpointBindingRecordFrameLength::decode_prefix(bytes); }",
        "fn read() { DurableSegmentManifest::decode(bytes, format); }",
    ] {
        assert!(!check_source(NEW_CONSUMER, source).is_empty(), "accepted {source}");
    }
}

#[test]
fn comments_strings_builders_and_test_only_items_are_not_production_routes() {
    assert!(check_source(
        NEW_CONSUMER,
        r#"
        // BootstrapCatalog::decode must stay behind admission.
        const NOTE: &str = "decode_extent_chunk";
        fn write() { BootstrapCatalog::builder(); }
        #[cfg(test)] mod tests { fn decode() { BootstrapCatalog::decode(bytes); } }
        #[cfg(test)] fn fixture() { decode_inline_record(bytes); }
    "#
    )
    .is_empty());
}

#[test]
fn writer_allowance_is_exact_in_path_and_operation() {
    let writer = "workspaces/worth-store/crates/worth-store/src/physical_runtime/record_serving/admission/initialization.rs";
    assert!(check_source(writer, "fn write() { durable_artifact_checksum(bytes); }").is_empty());
    assert!(!check_source(writer, "fn read() { BootstrapCatalog::decode(bytes); }").is_empty());
    assert!(!check_source(
        NEW_CONSUMER,
        "fn write() { durable_artifact_checksum(bytes); }"
    )
    .is_empty());
}

#[test]
fn newly_added_nested_source_is_automatically_governed() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("worth-c9-route-{}-{nonce}", std::process::id()));
    let crate_root = root.join("workspaces/worth-store/crates/worth-store");
    let source = root.join(NEW_CONSUMER);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, "fn read() { BootstrapCatalog::decode(bytes); }").unwrap();
    std::fs::write(
        crate_root.join("Cargo.toml"),
        "[package]\nname = 'worth-store'\nversion = '0.0.0'\n",
    )
    .unwrap();
    std::fs::write(
        crate_root.join("src/lib.rs"),
        "mod new_owner { mod new_reader; }",
    )
    .unwrap();
    let governed = GovernedCrate {
        package: "worth-store".into(),
        crate_root,
        relative_crate_root: "workspaces/worth-store/crates/worth-store".into(),
    };
    let graph = parse_crate_modules(&governed).unwrap();
    let violations = enforce(&governed, &graph);
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].subject().ends_with("new_owner/new_reader.rs"));
}

#[test]
fn payload_projection_requires_the_exact_private_family_method() {
    let owner = "workspaces/worth-store/crates/worth-store/src/physical_runtime/integrity/resident_admission/root_tree.rs";
    let valid = "impl IntegrityAdmittedResidentRootRoutingView<'_> { fn project_block(&self) { PhysicalRootRoutingBlock::project_payload(payload, identity, capacity, limits); } }";
    assert!(check_source(owner, valid).is_empty());
    assert!(!check_source(NEW_CONSUMER, valid).is_empty());
    for source in [
        "fn read() { PhysicalRootRoutingBlock::project_payload(payload, identity, capacity, limits); }",
        "impl OtherView { fn project_block(&self) { PhysicalRootRoutingBlock::project_payload(payload, identity, capacity, limits); } }",
        "impl IntegrityAdmittedResidentRootRoutingView<'_> { fn other(&self) { PhysicalRootRoutingBlock::project_payload(payload, identity, capacity, limits); } }",
        "impl IntegrityAdmittedResidentRootRoutingView<'_> { fn project_block(&self) { PhysicalSegmentMembershipBlock::project_payload(payload, identity, capacity, limits); } }",
        "fn read() { admitted.with_owner_decoder(context, |view| PhysicalRootRoutingBlock::decode(view.bytes(), capacity)); }",
    ] {
        assert!(!check_source(owner, source).is_empty(), "accepted {source}");
    }
}
