//! Real Stable aliases retain the performed output while replacing source facts.

use super::*;
use worth_query_consumer_values::{
    PlanarAdjustmentResult, PlanarCurrentOutputExpectation, PlanarDerivedOutput, PlanarOperation,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryCurrentOutputDenial,
    WorthQueryCurrentOutputDenialKind,
};

mod differential;

#[test]
fn installed_alias_full_oracle_preserves_output_and_rejects_later_output_change() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let performed = (0..256)
        .find_map(|_| match a.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("A performs through the installed producer");
    assert_eq!(
        performed.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    let performed_receipt = performed.application_commit_receipt().unwrap().clone();
    let downstream = (0..256)
        .find_map(|_| match b.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("B actually consumes performed A before either alias exists");
    assert_eq!(downstream.producer_contacts_in_this_demand(), 1);

    for (replacement_y, idempotency) in [(2, 0x9176_3101_u64), (3, 0x9176_3102)] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-b".to_owned(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-b".to_owned(),
                replacement_y: length(replacement_y),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(&application)
            .unwrap();
        drop(source);
        let mut refresh = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .controls(input_cutoff::controls())
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
            .expect("ordinary fresh admission selects the changed source");
        let before = request.retain_read().unwrap();
        let alias = (0..256)
            .find_map(|_| match refresh.advance(&request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            })
            .expect("fresh admission publishes the next real Stable alias");
        let after = request.retain_read().unwrap();
        assert_eq!(
            alias.posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(alias.producer_contacts_in_this_demand(), 0);
        assert!(alias.application_commit_receipt().is_none());
        // The same advance consumed B's queued source change. That is the
        // only commit: B then settles on it without contact or new commit.
        assert_ne!(before.selected_commit(), after.selected_commit());
        let WorthQueryApplicationOutputDemandProgress::Settled(consumer) =
            b.advance(&request).unwrap()
        else {
            panic!("B was progressed inside the alias advance");
        };
        // B's open demand initiated its initial execution; this advance adds none.
        assert_eq!(consumer.producer_contacts_in_this_demand(), 1);
        assert_eq!(
            consumer.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        assert_eq!(
            request.retain_read().unwrap().selected_commit(),
            after.selected_commit()
        );
        // The planar contract declares one fixed role and one family; the
        // alias carries both exactly as the performed commit bound them.
        let alias_outputs = alias.outputs_of::<PlanarOutputs>().unwrap();
        let performed_outputs = performed_receipt.outputs_of::<PlanarOutputs>().unwrap();
        assert_eq!(
            alias_outputs
                .entity::<PlanarAnchorOutput<CheckpointSchema>>()
                .unwrap()
                .entity_id(),
            performed_outputs
                .entity::<PlanarAnchorOutput<CheckpointSchema>>()
                .unwrap()
                .entity_id()
        );
        assert_eq!(
            alias_outputs
                .family_entries::<PlanarCreatedOutputs<CheckpointSchema>>()
                .unwrap(),
            performed_outputs
                .family_entries::<PlanarCreatedOutputs<CheckpointSchema>>()
                .unwrap()
        );
        assert!(
            matches!(
                mutate_anchor(&request, &application, output_probe(), idempotency + 10),
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. }),
            ),
            "current_output must accept the performed output through every real alias"
        );
    }

    assert!(matches!(
        mutate_anchor(
            &request,
            &application,
            PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-a".to_owned(),
                value: length(99),
            }),
            0x9176_3120
        ),
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. }),
    ));
    let refused = mutate_anchor(&request, &application, output_probe(), 0x9176_3121);
    let Err(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(denial),
    )) = refused
    else {
        panic!("the changed output must fail at the current-output verification boundary");
    };
    let denial = denial.downcast::<WorthQueryCurrentOutputDenial>().unwrap();
    assert_eq!(
        denial.kind(),
        WorthQueryCurrentOutputDenialKind::StaleSource
    );
    drop((a, b));
}

fn output_probe() -> PlanarOperation {
    PlanarOperation::VerifyCurrentOutputs(vec![PlanarCurrentOutputExpectation {
        producer_key: "anchor-a".to_owned(),
        output_key: "anchor-a".to_owned(),
    }])
}

fn mutate_anchor<'application>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
    application: &'application application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    operation: PlanarOperation,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<
        worth_query_consumer_values::PlanarMutationDenial,
        PlanarAdjustmentResult,
    >,
    WorthQueryApplicationRequestMutationDenial,
> {
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation,
            validator_work: 4_096,
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&idempotency)
        .execute_in_program::<program::ChainProgram>(application)
}
