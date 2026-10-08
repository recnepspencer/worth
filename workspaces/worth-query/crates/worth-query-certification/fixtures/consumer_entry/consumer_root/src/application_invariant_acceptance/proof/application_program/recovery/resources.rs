use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{PlanarOutputRead, PlanarRead, PlanarSourceAdjustment};

use super::{authentication, installation, length, output_controls, settle, ConsumerSchema};

pub(in super::super) fn resource_denial_preserves_source_and_delivery(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source_result = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = source_result.observed_sources()[0].clone();
    let denied_controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    );
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-c".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_003)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the source operation reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication must succeed before derived admission")
    };
    let failure = performed
        .start_required_outputs(&request, denied_controls)
        .err()
        .expect("insufficient derived resources deny required-output start");
    assert_eq!(
        failure.denial().recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Retryable,
        "required-output start denial: {:?}",
        failure.denial(),
    );
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = failure.denial()
    else {
        panic!("resource denial must preserve the exact output-demand cause")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(failure.result().changed_vertices, 1);
    let _committed_source = failure
        .receipt()
        .committed_product_publication()
        .composite_commit();
    let mut retried = failure
        .into_performed()
        .start_required_outputs(&request, output_controls())
        .unwrap_or_else(|failure| panic!("exact prepared retry starts: {:?}", failure.denial()));
    let published = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the successful source publication remains visible");
    assert_eq!(published.rows()[0].y, length(2));
    let settled = settle(
        || match retried.required_output_mut().advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        },
    );
    let row = request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the recovered output is retained");
    assert_eq!(row.rows()[0].value, length(3));
}
