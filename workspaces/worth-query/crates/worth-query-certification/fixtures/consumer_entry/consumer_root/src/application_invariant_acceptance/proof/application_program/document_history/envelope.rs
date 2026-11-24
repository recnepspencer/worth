//! A recorded envelope refuses a stage whose tree reaches an external producer, before any effect.
use super::*;
use crate::application_program::historical::eligibility;
use worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
    PlanarSourceAdjustment,
};

fn install_external() -> installation::ConsumerWorld<eligibility::Program> {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, eligibility::Program>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap())
        .validated_program()
        .unwrap();
    installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        program,
    )
    .unwrap()
}

/// The stage's own operation is ordinary. Its required tree reaches a
/// producer with an installed external effect, which no undo could take back.
#[test]
fn a_stage_whose_tree_reaches_an_external_producer_refuses_before_any_effect() {
    let world = install_external();
    let application = &world.application;
    let (document, branch) = attach_with_branch(application, "anchor-a");
    let before = with_request(application, |request| {
        document.current_step(request).unwrap()
    });
    let contacts = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&contacts);
    let executed = with_branch_request(application, branch, |request| {
        let mut envelope = document
            .edit_envelope(application, request, 0x9182_e200)
            .unwrap_or_else(|failure| panic!("envelope construction: {:?}", failure.denial()));
        envelope
            .push_stage::<eligibility::Root, _>(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(2),
            })
            .unwrap_or_else(|denial| panic!("stage capture: {denial:?}"));
        envelope.execute_recorded(
            WorthQueryHistoryStepMetadata::from_bytes(Vec::new()),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    });
    let (document, denial) = match executed {
        Err(failure) => failure.into_parts(),
        Ok(_) => panic!("an external producer must refuse"),
    };
    match denial {
        WorthQueryHistoryEditEnvelopeDenial::Stage {
            ordinal: 0,
            denial: WorthQueryDocumentHistoryDenial::ExternalEffectProducer { producer },
        } => assert_eq!(
            producer,
            "worth.query.certification.escaping-planar-producer.v1"
        ),
        other => panic!("the refusal must name the external producer: {other:?}"),
    }
    assert_eq!(planar_source_adjustment_contacts(&contacts), (0, 0));
    with_request(application, |request| {
        assert_eq!(document.current_step(request).unwrap().id(), before.id());
        assert_eq!(document.read(request).unwrap().rows()[0].value, length(1));
    });
    close(document, application);
}
