//! Certified Native byte identity through ordinary edits and recovery.
use super::model::{authentication::*, edit::Value, host, schema::*};
use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
mod controls;
mod native;
mod predicate;
mod write_admission;
use native::{assert_wire, CertifiedObservation};

const FIVE: [u8; 9] = [1, 5, 0, 0, 0, 0, 0, 0, 0];
const SEVEN: [u8; 9] = [1, 7, 0, 0, 0, 0, 0, 0, 0];
const ELEVEN: [u8; 9] = [1, 11, 0, 0, 0, 0, 0, 0, 0];
const FORTY_ONE: [u8; 9] = [1, 41, 0, 0, 0, 0, 0, 0, 0];

#[test]
fn inline_domain_edits_and_retained_reads_keep_exact_native_bytes() {
    let (application, authority) = host::install_with_projection();
    let initial = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&initial.primary(&application), &FIVE);
    host::edit(&application, Value::Primary(7), "inline-first");
    let first_pin = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&first_pin.primary(&application), &SEVEN);
    host::edit(&application, Value::Primary(11), "inline-patch");
    let patched = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&patched.primary(&application), &ELEVEN);
    assert_wire(&initial.primary(&application), &FIVE);
    assert_wire(&first_pin.primary(&application), &SEVEN);
    assert_wire(&patched.primary(&application), &ELEVEN);
}

#[test]
fn inline_domain_recovery_preserves_native_digest_material() {
    let (application, authority) = host::install_with_projection();
    host::edit(&application, Value::Primary(41), "inline-memory-forty-one");
    let before = CertifiedObservation::capture(&application, &authority, "numeric");
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
    // Checkpoint metadata includes runtime identities which legitimately rebind.
    // Compare complete model bytes only within each runtime; payload and digest
    // identity across rebuild are certified independently by admitted reads.
    let (dir_app, dir_authority) =
        host::open_directory_with_projection(host::unique_directory("inline-domain-digest"));
    let dir_scope = request_scope();
    let dir_principal = authenticate_operator(dir_app.installed_schema(), &dir_scope);
    let outcome = dir_app
        .runtime()
        .request(&dir_principal, &dir_scope)
        .advanced()
        .mutate(super::model::edit::Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(41),
        })
        .without_source()
        .idempotency(&"inline-directory-forty-one".to_owned())
        .execute_in_program(
            &dir_app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "inline directory edit must commit: {outcome:?}"
    );
    drop((outcome, dir_principal, dir_scope));
    let dir_before = CertifiedObservation::capture(&dir_app, &dir_authority, "numeric");
    assert_wire(&dir_before.primary(&dir_app), &FORTY_ONE);
    let dir_current = native::stable_current(&dir_app, "numeric");
    assert_eq!(native::digest(&dir_current), original_digest);
    drop((dir_before, dir_authority));
    let directory = dir_app.checkpoint_and_close_for_reopen();
    let restored = host::open_directory(directory);
    let value = native::stable_current(&restored, "numeric");
    assert_wire(&value, &FORTY_ONE);
    assert_eq!(native::digest(&value), original_digest);
    assert_eq!(
        prepare_aspect_value_identity_basis(&value).as_str(),
        literal
    );
}


fn head<
    P: worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
        NumericSchema,
    >,
>(
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<NumericSchema, P>,
) -> impl Eq + std::fmt::Debug + use<P> {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap()
}
