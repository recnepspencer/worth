#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::authority_sealing::reopen_boundary::*;
use tempfile::tempdir;
use worth_proof::TransitionOutcome;

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn reopen_cannot_consume_serialized_signal_state() {
    reopen_ignores_external_state(
        "worth-c5-serialized-signal-reopen",
        b"serialized Signal state",
        "serialized-signal-reopen",
    );
}

#[test]
fn ordinary_physical_work_cannot_add_an_internal_json_carrier() {
    reopen_ignores_external_state(
        "worth-c5-internal-physical-work-json",
        br#"{"route":"forged"}"#,
        "internal-json-carrier",
    );
}
