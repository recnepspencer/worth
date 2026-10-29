#[allow(unused_imports)] use crate::root_protocol_corruption_matrix::*;
use {std::fs};

use {worth_foundational::PhysicalIntegrityPosture};
use {worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::OfflineArtifactDuplicateEvidence, worth_store_offline_integrity_observer::OfflineArtifactObservation, worth_store_offline_integrity_observer::OfflineIntegrityOutcome, worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius, worth_store_offline_integrity_observer::OfflinePhysicalDamageCause, worth_store_offline_integrity_observer::OfflinePhysicalFormatField, worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason, worth_store_offline_integrity_observer::OfflineUnsupportedVersionAxis};

use {crate::root_protocol_expected_counters::assert_counters, crate::root_protocol_expected_counters::expected_counters};
use {crate::store_offline_integrity_observer_support::artifact_path, crate::support::fixtures::clean_store, crate::store_offline_integrity_observer_support::request, crate::store_offline_integrity_observer_support::Target};

#[test]
fn current_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Current);
}

#[test]
fn previous_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Previous);
}

#[test]
fn addressed_root_manifest_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Root);
}
