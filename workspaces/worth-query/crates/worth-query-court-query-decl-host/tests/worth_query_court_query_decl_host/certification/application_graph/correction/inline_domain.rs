//! Certified Native byte identity through the existing authored correction and recovery owners.
use super::model::{
    authentication::*, authored_identity_courts, edit::Value, history_reapplication, host,
    schema::*, tree_reapplication,
};
use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};
use worth_query_host::facade::{
    application_entry::*, application_installation::*, primary_graph::*,
};
mod controls;
mod native;
mod predicate;
use native::{assert_wire, capture, CertifiedObservation};

const FIVE: [u8; 9] = [1, 5, 0, 0, 0, 0, 0, 0, 0];
const SEVEN: [u8; 9] = [1, 7, 0, 0, 0, 0, 0, 0, 0];
const ELEVEN: [u8; 9] = [1, 11, 0, 0, 0, 0, 0, 0, 0];
const FORTY_ONE: [u8; 9] = [1, 41, 0, 0, 0, 0, 0, 0, 0];

#[test]
fn inline_domain_write_patch_correction_inverse_and_history_keep_exact_native_bytes() {
    let (application, authority) = host::install_with_projection();
    let initial = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&initial.primary(&application), &FIVE);
    let first = host::edit(&application, Value::Primary(7), "inline-first");
    let first_pin = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&first_pin.primary(&application), &SEVEN);
    let second = host::edit(&application, Value::Primary(11), "inline-patch");
    let patched = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&patched.primary(&application), &ELEVEN);
    tree_reapplication::reverse(&application, &second, "inline-inverse-patch");
    assert_wire(
        &CertifiedObservation::capture(&application, &authority, "numeric").primary(&application),
        &SEVEN,
    );
    let reversal = tree_reapplication::reverse(&application, &first, "inline-correct-first");
    assert_wire(
        &CertifiedObservation::capture(&application, &authority, "numeric").primary(&application),
        &FIVE,
    );
    history_reapplication::discard(&application);
    history_reapplication::rebuild(&application);
    let alternative = tree_reapplication::alternative(&application, &first, &reversal, 1);
    let replay = tree_reapplication::reapply(&application, &alternative, "inline-reapply");
    assert_ne!(replay.commit_reference(), first.commit_reference());
    assert_wire(
        &CertifiedObservation::capture(&application, &authority, "numeric").primary(&application),
        &SEVEN,
    );
    let head = super::model::required_output_courts::head(&application);
    let replayed = tree_reapplication::reapply(&application, &alternative, "inline-reapply");
    assert_eq!(replayed.commit_reference(), replay.commit_reference());
    assert_eq!(
        super::model::required_output_courts::head(&application),
        head
    );
    // Historical Native observations are read again after all inverses/replays.
    assert_wire(&initial.primary(&application), &FIVE);
    assert_wire(&first_pin.primary(&application), &SEVEN);
    assert_wire(&patched.primary(&application), &ELEVEN);
}

#[test]
fn inline_domain_retirement_restoration_and_recovery_preserve_native_digest_material() {
    let (application, authority) = host::install_with_projection();
    authored_identity_courts::create(&application, "inline-authored", None);
    let before = CertifiedObservation::capture(&application, &authority, "inline-authored");
    let original = before.primary(&application);
    assert_wire(&original, &FORTY_ONE);
    let literal = "value.kind#12:inline-bytes;value.inline-bytes#9:012900000000000000;";
    assert_eq!(
        prepare_aspect_value_identity_basis(&original).as_str(),
        literal
    );
    let original_digest = native::digest(&original);
    let corrupted = AspectValue::InlineBytes(
        worth_foundational::facade::InlineBytes::try_new(vec![1, 42, 0, 0, 0, 0, 0, 0, 0]).unwrap(),
    );
    assert_ne!(
        native::digest(&corrupted),
        original_digest,
        "a changed native payload cannot preserve the digest"
    );
    let retirement = authored_identity_courts::retire(&application, "inline-authored", false);
    authored_identity_courts::absent(&application, "inline-authored");
    assert_wire(&before.primary(&application), &FORTY_ONE);
    tree_reapplication::reverse(&application, &retirement, "inline-restore-retired");
    let restored_pin = CertifiedObservation::capture(&application, &authority, "inline-authored");
    assert_wire(&restored_pin.primary(&application), &FORTY_ONE);
    // Checkpoint metadata includes runtime identities which legitimately rebind.
    // Compare complete model bytes only within each runtime; payload and digest
    // identity across rebuild are certified independently by admitted reads.
    let original_current = native::stable_current(&application, "inline-authored");
    assert_eq!(native::digest(&original_current), original_digest);
    let checkpoint = capture(&application);
    let restored = host::restore(checkpoint);
    let value = native::stable_current(&restored, "inline-authored");
    assert_wire(&value, &FORTY_ONE);
    assert_eq!(native::digest(&value), original_digest);
    assert_eq!(
        prepare_aspect_value_identity_basis(&value).as_str(),
        literal
    );
}

#[test]
fn inline_domain_retired_output_compensation_and_composite_definition_stay_certified() {
    super::model::source_output_courts::run();
    super::model::composite_definition_court::run();
}

#[test]
fn inline_domain_inverse_restores_independent_native_revision_and_business_bytes() {
    use super::model::composite_definition_model::Change;
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let (application, authority) = host::install_with_projection();
    host::edit(&application, Value::Primary(7), "separate-business-atom");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = "separate-definition-revision".to_owned();
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Change {
            key: "numeric".into(),
            value: 23,
            revision: 29,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("independent value/revision source must commit: {outcome:?}");
    };
    assert_eq!(
        CertifiedObservation::capture(&application, &authority, "numeric").revision(&application),
        29
    );
    tree_reapplication::reverse(&application, &receipt, "separate-definition-inverse");
    assert_wire(
        &CertifiedObservation::capture(&application, &authority, "numeric").primary(&application),
        &SEVEN,
    );
    assert_eq!(
        CertifiedObservation::capture(&application, &authority, "numeric").revision(&application),
        5
    );
}
