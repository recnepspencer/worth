#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::managed_mutation::retirement_wal_hold::*;

use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare};
use {worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::assert_one_page_released_net_of_wal, worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::wal_bytes};

#[test]
fn unresolved_retirement_survives_wal_rotation_and_checkpoint() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let wal = wal_policy();
    let serving = open_initialized(&root, wal);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [71; 32], b"hold-source").execute());
    completed(prepare_rewrite(&serving, placement, [72; 32]).execute());
    serving.certification_stop_after_retirement_delete();
    assert!(serving.retire_displaced_segment().is_err());
    let mut rotated = false;
    for index in 0..8 {
        let byte = 80 + u8::try_from(index).unwrap();
        completed(prepare(&serving, placement, [byte; 32], b"rotate-tail").execute());
        if intent_is_behind_the_active_tail(&root) {
            rotated = true;
            break;
        }
    }
    assert!(
        rotated,
        "a later append must leave the retirement intent off the active tail"
    );
    let holders = retirement_wal_files(&root);
    assert!(!holders.is_empty());
    checkpoint(&serving);
    let after = wal_names(&root);
    assert!(
        holders.iter().all(|name| after.contains(name)),
        "checkpoint reclamation must keep the unresolved retirement segment"
    );
    serving.close();
    let serving = open_existing(&root, wal);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(serving.certification_charged_growth_bytes(), charged);
    let wal_before = wal_bytes(&root);
    serving.retire_displaced_segment().unwrap();
    assert_one_page_released_net_of_wal(
        &root,
        wal_before,
        charged,
        serving.certification_charged_growth_bytes(),
        page_bytes,
    );
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent)
    );
    let released = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(serving.certification_charged_growth_bytes(), released);
    serving.close();
}
