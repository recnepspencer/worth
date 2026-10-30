//! Independently compiled V2 schema; persisted names and scalar loci match V1.
pub use super::{model, output as wire_model};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
#[path = "successor/acknowledged_capture.rs"]
pub mod acknowledged_capture;
#[path = "successor/adoption.rs"]
pub mod adoption;
#[path = "../canonical_value/binding.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod binding;
#[path = "derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod derived_query;
#[path = "edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod edit;
#[path = "successor/historical.rs"]
pub mod historical;
#[path = "host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod host;
#[path = "successor/output.rs"]
pub mod output;
#[path = "successor/pre_effect_indexes.rs"]
pub mod pre_effect_indexes;
#[path = "producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod producer;
#[path = "program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod program;
#[path = "../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles these declarations and native handlers against its distinct V2 schema and codec"
)]
pub mod query;
#[path = "readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the successor compiles readiness against its actual schema and installed producer"
)]
pub mod readiness;
#[path = "successor/schema.rs"]
pub mod schema;
#[path = "successor/unchanged_revision.rs"]
pub mod unchanged_revision;

pub use installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
pub use worth_query_host::facade::{
    application_entry::*, application_installation as installation,
};

/// Exercise the real V2 Preserve producer after an admitted accepted output.
pub fn assert_edit_regenerates(application: &host::Host) {
    let scope =
        worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope();
    let principal = worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::authenticate_operator(
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

pub fn predecessor_checkpoint() -> installation::WorthQueryApplicationCheckpoint {
    let application = super::host::install(model::authored_value()).unwrap();
    let scope =
        worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope();
    let principal = worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::authenticate_operator(
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
