//! A genuine created canonical output reaches the unsupported native transition door.
//! These baseline cases preserve native source identity and decoded values.
//! Genuine codec adoption still requires an independent full stored authored
//! field-byte/carrier oracle, alongside identity and value preservation.
#[path = "canonical_value/binding.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
mod binding;
#[path = "canonical_transition/derived_query.rs"]
mod derived_query;
#[path = "canonical_transition/edit.rs"]
mod edit;
#[path = "canonical_transition/host.rs"]
mod host;
#[path = "canonical_value/model.rs"]
#[expect(dead_code, reason = "wire golden is qualified by canonical_value")]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
mod model;
#[path = "canonical_transition/output.rs"]
mod output;
#[path = "canonical_transition/producer.rs"]
mod producer;
#[path = "canonical_transition/program.rs"]
mod program;
#[path = "canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
mod query;
#[path = "canonical_transition/readiness.rs"]
mod readiness;
#[path = "canonical_transition/related.rs"]
mod related;
#[path = "canonical_transition/schema.rs"]
mod schema;
#[path = "canonical_transition/successor.rs"]
mod successor;

use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{application_entry::*, application_installation as installation};

#[test]
fn accepted_created_canonical_output_refuses_unsupported_transition_without_changing_source() {
    let authored = model::authored_value();
    assert!(!authored.children.is_empty());
    let host = host::install(authored.clone()).unwrap();
    let source_id = host::source_id(&host);
    let expected = output::calculate(&authored);
    assert_eq!(expected.caption, "ASSEMBLY Β");
    assert_eq!(expected.rank_sum, 12);
    assert_eq!(expected.tags, ["left", "right"]);
    assert_eq!(host::read(&host), authored);
    let runtime = host.runtime();
    let retained_catalogue = host.installed_schema().value_bindings().clone();
    assert!(retained_catalogue.same_installation_as(host.installed_schema().value_bindings()));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world());
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the real installed producer must settle");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let created = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap();
    let created_id = created.entity_id();
    assert_eq!(host::read_derived(&host), expected);
    assert_eq!(
        host::source_id(&host),
        source_id,
        "created output must preserve the native authored source entity"
    );
    assert_eq!(
        host::read(&host),
        authored,
        "derived publication must preserve authored nested source"
    );
    let (checkpoint, sections) = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &host::program().revision().to_string(),
    )
    .unwrap();
    let mut callback_ran = false;
    let denied = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host::limits(),
        checkpoint.clone(),
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(512, 32, 16384).unwrap(),
        CapturePolicy::SystemAllocation,
        |_, _| {
            callback_ran = true;
            Ok(())
        },
    )
    .err()
    .expect("unmapped actual accepted output must refuse transition");
    assert!(!callback_ran, "refusal must precede callback authoring");
    assert!(format!("{denied:?}").contains("accepted-output migration support"));
    drop(demand);
    drop(host);
    let restored = host::restore(checkpoint);
    assert!(!retained_catalogue.same_installation_as(restored.installed_schema().value_bindings()));
    assert_eq!(
        host::source_id(&restored),
        source_id,
        "current-format restore must preserve the native authored source entity"
    );
    assert_eq!(host::read(&restored), authored);
    assert_eq!(host::read_derived(&restored), expected);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(restored.installed_schema(), &fresh_scope);
    let fresh = restored
        .runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(restored.runtime().current_world());
    let mut reopened = fresh
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(reused) =
        reopened.settle(&fresh).unwrap()
    else {
        panic!("restored native output settles");
    };
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        reused
            .outputs_of::<output::DerivedOutputs>()
            .unwrap()
            .entity::<output::CreatedOutput>()
            .unwrap()
            .entity_id(),
        created_id
    );
}

#[test]
fn derived_successor_wire_is_incompatible_while_authored_source_codec_is_unchanged() {
    use worth_foundational::facade::{
        decode_canonical_value, encode_canonical_value, ValueDecodeBounds,
    };
    let bounds = ValueDecodeBounds::new(8192, 512, 32, 12);
    let v1 = output::calculate(&model::authored_value());
    let wire = encode_canonical_value(&v1, bounds).unwrap();
    assert!(decode_canonical_value::<output::DerivedV2>(&wire, bounds).is_err());
    assert_eq!(
        decode_canonical_value::<output::DerivedV1>(&wire, bounds).unwrap(),
        v1
    );
}
