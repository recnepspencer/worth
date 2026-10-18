//! Independently compiled V2 schema; persisted names and scalar loci match V1.
use super::{model, output as wire_model};
#[path = "successor/acknowledged_capture.rs"]
mod acknowledged_capture;
#[path = "successor/adoption.rs"]
mod adoption;
#[path = "../canonical_value/binding.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod binding;
#[path = "derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod derived_query;
#[path = "edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod edit;
#[path = "host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod host;
#[path = "successor/output.rs"]
mod output;
#[path = "successor/pre_effect_indexes.rs"]
mod pre_effect_indexes;
#[path = "producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod producer;
#[path = "program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod program;
#[path = "../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
mod query;
#[path = "readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles readiness against its actual schema and installed producer"
)]
mod readiness;
#[path = "successor/schema.rs"]
mod schema;
#[path = "successor/unchanged_revision.rs"]
#[expect(
    dead_code,
    reason = "the compiled missing-revision twin exercises recovery refusal, not every shared host entry"
)]
mod unchanged_revision;

use installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::{application_entry::*, application_installation as installation};

#[test]
fn actual_v2_installation_produces_and_reads_its_supported_incompatible_output() {
    let authored = model::authored_value();
    let application = host::install(authored.clone()).unwrap();
    let source_id = host::source_id(&application);
    output::reset_decodes();
    let scope = crate::document_retention_model::operator_identity::request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_operator(
        application.installed_schema(),
        &scope,
    );
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.runtime().current_world());
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("real V2 producer must settle");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let derived = host::read_derived(&application);
    assert_eq!(derived.0.caption, "ASSEMBLY Β");
    assert_eq!(derived.0.total_rank, 12);
    assert_eq!(derived.0.labels, ["left", "right"]);
    assert!(
        output::decodes() > 0,
        "ordinary installed V2 read must enter its actual decoder"
    );
    assert_eq!(host::read(&application), authored);
    assert_eq!(host::source_id(&application), source_id);
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(demand);
    drop(application);
    let restored = host::restore(checkpoint);
    assert_eq!(host::source_id(&restored), source_id);
    assert_eq!(host::read_derived(&restored), output::calculate(&authored));
    assert_eq!(host::read(&restored), authored);
    assert_edit_regenerates(&restored);
}

/// Exercise the real V2 Preserve producer after an admitted accepted output.
pub fn assert_edit_regenerates(application: &host::Host) {
    let scope = crate::document_retention_model::operator_identity::request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_operator(
        application.installed_schema(),
        &scope,
    );
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.runtime().current_world());
    let mut original = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        original.settle(&request).unwrap()
    else {
        panic!("the accepted V2 output must settle before editing its source");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 0);
    drop(original);
    assert_edit_preserves_settled_output(application, &settled);
}

/// Carry the actual created output into an ordinary source edit and Preserve.
pub fn assert_edit_preserves_settled_output(
    application: &host::Host,
    settled: &WorthQueryApplicationOutputDemandSettlement<query::ValueQuery>,
) {
    let source_id = host::source_id(application);
    let original_id = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap()
        .entity_id();
    let mut edited = model::authored_value();
    edited.title = "regenerated γ".to_owned();
    edited.children[0].rank = 40;
    edited.children[1].rank = -3;
    edited.children[0].tag.0 = "updated-left".to_owned();
    let regenerated = host::edit_source(application, edited.clone(), 0x271_0201);
    assert_eq!(host::read(application), edited);
    let preserved_id = regenerated
        .outputs_of::<output::PreservedOutputs>()
        .unwrap()
        .entity::<output::PreservedOutput>()
        .unwrap()
        .entity_id();
    assert_eq!(
        preserved_id, original_id,
        "regeneration must preserve the actual prior output entity"
    );
    let derived = host::read_derived(application);
    assert_eq!(derived.0.caption, "REGENERATED Γ");
    assert_eq!(derived.0.total_rank, 37);
    assert_eq!(derived.0.labels, ["updated-left", "right"]);
    assert_eq!(host::source_id(application), source_id);
    assert_eq!(host::read(application), edited);
}

#[test]
fn exact_v2_restore_requires_explicit_transition_before_any_derived_decode() {
    let checkpoint = predecessor_checkpoint();
    output::reset_decodes();
    let denial = installation::in_memory_program_from_checkpoint(
        host::program(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        host::limits(),
        checkpoint.clone(),
    )
    .err()
    .expect("incompatible recovered V1 field cannot enter V2 ordinary restore");
    assert_eq!(
        output::decodes(),
        0,
        "codec fence must precede V2 payload interpretation"
    );
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denial else {
        panic!("expected native checkpoint graph refusal: {denial:?}");
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
    assert!(
        denial
            .subject()
            .contains("recovered derived codec requires authenticated native transition admission"),
        "unexpected recovery fence: {denial:?}"
    );
    unchanged_revision::assert_missing_revision_refused(checkpoint);
}

fn predecessor_checkpoint() -> installation::WorthQueryApplicationCheckpoint {
    let application = super::host::install(model::authored_value()).unwrap();
    let scope = crate::document_retention_model::operator_identity::request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_operator(
        application.installed_schema(),
        &scope,
    );
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.runtime().current_world());
    let mut demand = request
        .demand(super::producer::Demand)
        .start_in_program::<_, super::program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("predecessor native producer must settle");
    };
    assert!(settled.application_commit_receipt().is_some());
    let (checkpoint, sections) = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    checkpoint
}
