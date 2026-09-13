use super::child_process::{run_courtroom_reopener, run_courtroom_writer};
use super::courtroom_evidence_support::{
    hex, inline_placement, locator_identities, offline_completion, placement_identities,
    reopener_completion, segment_files, segment_page_bytes, writer_completion,
};
use super::scenario_evidence::ScenarioProcessEvidence;

#[test]
fn record_world_survives_fresh_processes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let locators = parent.path().join("sealed-locators.csv");
    let oracle_path = parent.path().join("sealed-workload.csv");
    let oracle = super::courtroom_oracle::seal(&oracle_path);
    let writer_stdout = run_courtroom_writer(&root, &locators, &oracle_path);
    let writer = ScenarioProcessEvidence::parse_child(&writer_stdout, "writer");
    let writer_completion = writer_completion(&writer_stdout);
    let oracle_payload_digest = super::courtroom_oracle::payload_digest(&locators, &oracle);
    let oracle_point_digest = super::courtroom_oracle::point_digest(&locators, &oracle);
    let oracle_scan_digest = super::courtroom_oracle::scan_digest(&locators, &oracle);
    let oracle_records = oracle.len();
    assert_eq!(
        super::courtroom_oracle::locator_count(&locators),
        oracle_records
    );

    let reopener_stdout = run_courtroom_reopener(&root, &locators);
    let reopener = ScenarioProcessEvidence::parse_child(&reopener_stdout, "reopener");
    let reopened = reopener_completion(&reopener_stdout);

    let observer_stdout = super::observer::run(&root);
    let observer = ScenarioProcessEvidence::offline_process(&observer_stdout);
    let offline_fields = offline_completion(&observer_stdout);
    let format = super::scenario_configuration::courtroom_configuration().0;
    let walk = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    let first_locator = std::fs::read_to_string(&locators)
        .unwrap()
        .lines()
        .next()
        .map(super::child_process::decode_locator)
        .unwrap();
    let first_identity = (
        first_locator.encode()[16..32].try_into().unwrap(),
        u64::from_le_bytes(first_locator.encode()[32..40].try_into().unwrap()),
    );
    let prior = worth_store_offline_verifier::walk_non_current_durable_record_manifest(
        &root,
        format.declaration(),
        2,
    )
    .unwrap();
    let current_residue_refused = matches!(
        worth_store_offline_verifier::walk_non_current_durable_record_manifest(
            &root,
            format.declaration(),
            walk.root_generation(),
        ),
        Err(worth_store_offline_verifier::OfflineDurableManifestDenial::CurrentRootRequestedAsResidue)
    );
    assert!(current_residue_refused);
    let prior_first = inline_placement(&prior, first_identity);
    let current_first = inline_placement(&walk, first_identity);

    assert_eq!(writer_completion.root_generation, reopened.root_generation);
    assert_eq!(writer_completion.root_generation, walk.root_generation());
    assert_eq!(reopened.records, oracle_records);
    assert_eq!(reopened.records, walk.placements().len());
    assert_eq!(reopened.deferred_records, 1);
    assert_eq!(reopened.point_digest, oracle_point_digest);
    assert_eq!(reopened.scan_digest, oracle_scan_digest);
    assert_eq!(offline_fields.records, oracle_records);
    assert_eq!(
        offline_fields.root_generation,
        writer_completion.root_generation
    );
    assert_eq!(offline_fields.payload_digest, oracle_payload_digest);
    assert_eq!(offline_fields.payload_digest, hex(&walk.payload_digest()));
    assert!(segment_files(&root) >= 3);
    assert!(walk.manifest_blocks() > 1);
    assert!(segment_page_bytes(&root) >= 64 * 65_536);
    let locator_identity_set = locator_identities(&locators);
    let placement_identity_set = placement_identities(&walk);
    assert_eq!(placement_identity_set, locator_identity_set);
    assert_ne!(writer_completion.positioned_writes, 0);
    assert!(writer_completion.file_barriers >= writer_completion.positioned_writes);
    assert!(writer_completion.directory_barriers >= writer_completion.file_barriers);
    assert_eq!(writer_completion.catalog_replacements, 12);
    let extent_placements = walk
        .placements()
        .iter()
        .filter(|placement| {
            matches!(
                placement,
                worth_store_offline_verifier::OfflineRecordPlacement::Extent { .. }
            )
        })
        .count();
    assert_eq!(extent_placements, 2);
    assert_eq!(prior_first.slot, current_first.slot);
    assert_eq!(prior_first.page, current_first.page);
    assert!(current_first.page_generation > prior_first.page_generation);

    super::scenario_evidence::assert_distinct_processes(&[writer, reopener, observer]);
}
