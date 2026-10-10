//! Current-basis selection frees a performed observation no caller holds.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    PreparedOutputRootKind, WorthQueryPerformedOutputDemandSource,
};
use crate::domain_computation::primary_graph::tests::{application_attempt as attempts, fixture};

#[test]
fn current_commit_basis_reclaims_an_abandoned_performed_observation() {
    let world = fixture::installed_authorization_world(true);
    let request = fixture::live_scope();
    let principal = attempts::authenticated_principal(&world, &request);
    let account = attempts::resolved_account(&world, "open", &request);
    let program = attempts::preimage_evidence::retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "observation-release",
        attempts::preimage_evidence::RetentionMutationBreadth::Narrow,
    )
    .with_output_demand_observation();
    let WorthQueryApplicationCommitOutcome::Committed(mut receipt) =
        world.application.compare_and_commit_application(
            program,
            attempts::idempotency(241, 242),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the real performed write must commit");
    };
    let occurrence = receipt.product_branch().occurrence();
    let change = std::sync::Arc::new(receipt.take_performed_relational_product_change().unwrap());
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation()
        .unwrap();
    let retained_receipt = receipt.clone();
    let registry = &world.application.output_demands;
    let preparation = registry.begin_source_preparation(occurrence);
    let kind = PreparedOutputRootKind::Required(std::any::TypeId::of::<()>());
    let commit = registry
        .retain_performed_source(
            WorthQueryPerformedOutputDemandSource {
                receipt,
                change,
                observation,
                output_source_identity: None,
            },
            &preparation,
            kind.clone(),
            None,
        )
        .unwrap();
    registry.release_prepared_token(&commit);
    drop(preparation);
    let runtime = world.application.product_runtime();
    let retained = runtime.admit_product_occurrence(occurrence).unwrap();
    let mut held = Vec::new();
    loop {
        match runtime.admit_product_occurrence(occurrence) {
            Ok(product) => held.push(product),
            Err(
                crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted,
            ) => break,
            Err(denial) => panic!("unexpected product refusal: {denial:?}"),
        }
    }
    registry
        .validate_recovery_root_kind(&retained_receipt, kind.clone())
        .unwrap();
    world.application.with_application_advancement(&request, |_phase| {
        let current = select_current_product(&world.application, &retained)
            .expect("current commit selection reclaims the abandoned observation");
        assert_eq!(current.selected_commit(), retained.selected_commit());
        assert_eq!(registry.validate_recovery_root_kind(&retained_receipt, kind).unwrap_err().kind(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
        assert!(matches!(runtime.admit_product_occurrence(occurrence),
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted)),
            "the reclaimed slot funds the current basis while other observers stay held");
    }).unwrap();
}
