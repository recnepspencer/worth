//! Genuine performed mixed Preserve/Create/Retire readiness and recovery.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationPerformedMutationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::{application_contribution, primary_graph};

mod producer;
mod program;
mod readiness;
use producer::*;
use program::{MixedProgram, MixedRoot, MixedSchema};
mod installation;
use installation::install;

pub(crate) fn contracts<Schema: TopologySchemaBinding>(
    contracts: &mut application_contribution::WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    if std::any::TypeId::of::<Schema>() != std::any::TypeId::of::<MixedSchema>() {
        return Ok(());
    }
    contracts.producer::<ReplacementProducer<Schema>>()?;
    contracts.conditional::<readiness::ReplacementReadiness<Schema>>()?;
    Ok(())
}
pub(crate) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut application_contribution::WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    if std::any::TypeId::of::<Schema>() != std::any::TypeId::of::<MixedSchema>() {
        return Ok(());
    }
    setup.producer::<ReplacementProducer<Schema>>(ReplacementProvider)?;
    setup.conditional::<readiness::ReplacementReadiness<Schema>>(())
}

#[test]
fn mixed_retirement_performed_product_settles_and_checkpoint_stays_ineligible() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let performed = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_7302_u64)
        .execute_performed::<MixedProgram, MixedRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("real source command must perform")
    };
    let mut outputs = performed
        .start_required_outputs(&application, &request, Default::default())
        .unwrap_or_else(|failure| panic!("actual source output custody: {:?}", failure.denial()));
    let settled = (0..64)
        .find_map(|_| {
            match outputs
                .required_output_mut()
                .advance(&application, &request)
                .unwrap()
            {
                WorthQueryApplicationProgramOutputProgress::Pending => None,
                WorthQueryApplicationProgramOutputProgress::Settled(value) => Some(value),
            }
        })
        .expect("actual mixed replacement must settle as current");
    let root = settled
        .root_receipt()
        .expect("actual replacement retains its commit");
    let correspondence = root.outputs_of::<VertexReplacementOutputs>().unwrap();
    let preserved = correspondence
        .entity::<VertexReplacementAnchorOutput<MixedSchema>>()
        .unwrap()
        .entity_id();
    let created = correspondence
        .entity::<VertexReplacementCreatedOutput<MixedSchema>>()
        .unwrap()
        .entity_id();
    let retired = correspondence
        .entity::<VertexReplacementRetiredOutput<MixedSchema>>()
        .unwrap()
        .entity_id();
    assert_ne!(created, retired);
    assert_ne!(preserved, retired);
    let retained = request.retain_read().unwrap();
    request
        .at(&retained)
        .require_current_program_output(&settled, NonZeroUsize::new(4096).unwrap())
        .unwrap();

    assert!(
        request
            .query(PlanarRead {
                body_key: "anchor-b".into()
            })
            .execute()
            .is_err(),
        "the actually retired body has no live query view"
    );
    for key in ["anchor-a", "replacement-b"] {
        request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .expect("preserved and created members stay live");
    }
    let bytes = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap()
        .bytes()
        .to_vec();
    // The created live member still owns all installed aspects. Reverting its
    // Length value cannot revert the native revision or revive this settlement.
    for (command, value) in [(0x9176_7303_u64, 13), (0x9176_7304_u64, 1)] {
        let observed = request
            .query(PlanarRead {
                body_key: "replacement-b".into(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "replacement-b".into(),
                operation: worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(
                    worth_query_consumer_values::PlanarDerivedOutput {
                        body_key: "replacement-b".into(),
                        value: length(value),
                    },
                ),
            }))
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&command)
            .execute_in_program(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let changed = request.retain_read().unwrap();
        assert!(
            matches!(request.at(&changed).require_current_program_output(&settled, NonZeroUsize::new(4096).unwrap()),
            Err(WorthQueryProgramOutputCurrentnessDenial::Output(denial)) if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }
    request
        .at(&retained)
        .require_current_program_output(&settled, NonZeroUsize::new(4096).unwrap())
        .expect("the original retained observation remains exact");
    drop(retained);
    drop(settled);
    drop(outputs);
    drop(source);
    drop(principal);
    drop(scope);
    drop(application);
    reset_contacts();
    let reopened = install(Some(
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
    ));
    let (scope, principal) = authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let mut demand = request
        .demand(ReplacementDemand("anchor-a".into()))
        .start_in_program::<MixedProgram, MixedRoot>(&reopened)
        .unwrap();
    let denial = (0..64)
        .find_map(|_| match demand.advance(&request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Pending) => None,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_)) => {
                panic!("untrusted retired descriptions must not issue restored readiness")
            }
            Err(denial) => Some(denial),
        })
        .expect("ineligible retirement reuse attempts the ordinary producer");
    assert!(
        matches!(denial, worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause)
        if cause.kind() == WorthQueryOutputDemandDenialKind::ProducerUnavailable)
    );
    assert!(
        contacts() > 0,
        "this is ordinary Fresh execution, never restored retirement authority"
    );
}
