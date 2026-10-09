//! Mixed reconstruction retains its upstream source; all-retained output has no payload to suspend.
use super::super::{
    length, output_correspondence::observed_source, source_version, ProgramApplication, Request,
};
use crate::ConsumerSchema;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationPerformedMutationOutcome, WorthQueryOutputDemandControls,
    },
    application_invariants::EntityId,
    primary_graph::{
        WorthQueryGeneratedOutputSuspensionDenial, WorthQueryGeneratedOutputSuspensionFailure,
    },
};
use worth_query_topology_entry::{
    FinalPlanarOutputs, FinalRetainedSourceOutput, PlanarFinalPreserveProducer, PlanarOutputDemand,
    PlanarOutputToFinalConnection, PlanarSourceAdjustment,
};

pub(super) fn initial_source_identity(
    request: &Request<'_>,
    application: &ProgramApplication,
) -> EntityId {
    let mut output = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            PlanarOutputDemand::new("anchor-b"),
            controls(),
        )
        .expect("the installed current output is selected under fresh admission");
    let settlement =
        super::super::settle(|| super::super::settled(output.advance(request).unwrap()));
    let mut selected = settlement.outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>();
    let (_, final_output) = selected.next().expect("exact final producer output exists");
    assert!(selected.next().is_none());
    final_output
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalRetainedSourceOutput<ConsumerSchema>>()
        .unwrap()
        .entity_id()
}

pub(super) fn preserved_outputs_remain_read_only(
    application: &ProgramApplication,
    request: &Request<'_>,
    scope: &WorthQueryRequestScope,
    source_identity: EntityId,
) {
    assert_eq!(
        initial_source_identity(request, application),
        source_identity,
        "restoration must retain the original source identity outside generated custody"
    );
    assert_eq!(
        super::expected::read(request, "anchor-b").y,
        length(2),
        "reconstruction writes only generated payload, never the retained source"
    );
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".into(),
            replacement_y: length(3),
        })
        .expect_source(observed_source(request, "anchor-b"))
        .idempotency(&982)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the second source change must freshly publish")
    };
    let branch = performed.receipt().product_branch();
    let mut performed = performed
        .start_required_outputs(request, controls())
        .unwrap_or_else(|_| panic!("the actual preserve producer graph starts"));
    let settlement = super::super::settle(|| {
        super::super::settled(performed.required_output_mut().advance(request).unwrap())
    });
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .count(),
        1
    );
    drop(performed);
    let before = source_version(request);
    let source = observed_source(request, "anchor-b");
    match application
        .on_branch(branch)
        .select()
        .unwrap()
        .suspend_current_generated_output::<PlanarFinalPreserveProducer<ConsumerSchema>>(
            scope, source,
        ) {
        Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
            WorthQueryGeneratedOutputSuspensionDenial::NoGeneratedPayload,
        )) => {}
        _ => panic!("an all-retained output must refuse suspension without lifecycle effects"),
    }
    assert_eq!(
        source_version(request),
        before,
        "no-payload suspension cannot advance the journal"
    );
    for (key, y) in [
        ("final:anchor-b", 5),
        ("final:anchor-b:b", 5),
        ("final:anchor-b:c", 6),
    ] {
        assert_eq!(
            super::expected::read(request, key).y,
            length(y),
            "the actual preserved ring remains readable and unchanged by the refusal"
        );
    }
}

fn controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
}
