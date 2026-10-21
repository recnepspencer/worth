//! The same nonempty native carrier crosses a genuine derived-only codec boundary.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

#[test]
fn nonempty_v1_native_carrier_adopts_v2_then_produces_edits_and_reopens() {
    let authored = model::authored_value();
    assert!(!authored.children.is_empty());
    let predecessor = super::super::host::install(authored.clone()).unwrap();
    let source_id = super::super::host::source_id(&predecessor);
    let original_catalogue = predecessor.installed_schema().value_bindings().clone();
    let scope = request_scope();
    let principal = authenticate_operator(predecessor.installed_schema(), &scope);
    let request = predecessor
        .runtime()
        .request(&principal, &scope)
        .on_branch(predecessor.runtime().current_world());
    let mut demand = request
        .demand(super::super::producer::Demand)
        .start_in_program::<_, super::super::program::Root>(&predecessor)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the real installed V1 producer must publish");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let old_output = settled
        .outputs_of::<super::super::output::DerivedOutputs>()
        .unwrap()
        .entity::<super::super::output::CreatedOutput>()
        .unwrap()
        .entity_id();
    let (checkpoint, sections) = predecessor
        .runtime()
        .with_host_advancement(|phase| {
            predecessor
                .runtime()
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    let original_bytes = checkpoint.bytes().to_vec();
    let retained_checkpoint = checkpoint.clone();
    let expected_predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &super::super::host::program().revision().to_string(),
    )
    .unwrap();
    drop(demand);
    drop(predecessor);
    output::reset_decodes();
    let application = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host::limits(),
        checkpoint,
        expected_predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(4096, 32, 16384).unwrap(),
        CapturePolicy::SystemAllocation,
        |_, _| Ok(()),
    )
    .expect("native performed-field provenance admits same-carrier derived retirement/adoption");
    assert_eq!(
        retained_checkpoint.bytes(),
        original_bytes,
        "input checkpoint is never rewritten"
    );
    assert_eq!(output::decodes(), 0, "V2 never interprets V1 derived bytes");
    assert_eq!(host::source_id(&application), source_id);
    assert_eq!(host::read(&application), authored);
    assert!(
        !original_catalogue.same_installation_as(application.installed_schema().value_bindings())
    );
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(application.installed_schema(), &fresh_scope);
    let fresh = application
        .runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(application.runtime().current_world());
    let mut regenerated = fresh
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        regenerated.settle(&fresh).unwrap()
    else {
        panic!("the actual installed V2 producer must reconstruct the retired output");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let new_output = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap()
        .entity_id();
    assert_ne!(
        new_output, old_output,
        "historical output is never relabelled as V2"
    );
    let value = host::read_derived(&application);
    assert_eq!(value.0.caption, "ASSEMBLY Β");
    assert_eq!(value.0.total_rank, 12);
    assert_eq!(value.0.labels, ["left", "right"]);
    assert!(output::decodes() > 0);
    drop(regenerated);
    assert_edit_preserves_settled_output(&application, &settled);
    let current = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(application);
    let reopened = host::restore(current);
    assert_eq!(host::source_id(&reopened), source_id);
    let value = host::read_derived(&reopened);
    assert_eq!(value.0.caption, "REGENERATED Γ");
    assert_eq!(value.0.total_rank, 37);
    assert_eq!(value.0.labels, ["updated-left", "right"]);
    assert_eq!(retained_checkpoint.bytes(), original_bytes);
}
