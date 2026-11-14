//! A fixed unbound source cannot regain retired custody after reopen.
use super::directory_source_custody::open_ordinary;
use super::ordinary_source::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandDenial as Demand,
        WorthQueryRequiredOutputPreparationDenial as Preparation,
    },
    primary_graph::WorthQueryOutputDemandDenialKind,
};

#[test]
fn directory_replaced_unbound_source_stays_superseded_after_reopen() {
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let input = |days| SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: days,
        },
    };
    let mut values = [1,2,3].into_iter().filter(|days| *days !=
        worth_query_testkit_document_retention::facade::document_retention_model::host::SEED_RETENTION);
    let first = values.next().unwrap();
    let second = values.next().unwrap();
    let home = open_ordinary(&directory, true, execution);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let mut original = None;
        for (days, key) in [(first, 1_u64), (second, 2_u64)] {
            let outcome = request
                .advanced()
                .mutate(input(days))
                .without_source()
                .idempotency(&key)
                .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                    host,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let performed = match outcome {
                WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
                WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
                    panic!("source did not perform: {outcome:?}")
                }
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
                    panic!("source retention: {:?}", failure.denial())
                }
                _ => panic!("source has outstanding native custody"),
            };
            if days == first {
                original = Some(performed.receipt().clone());
            }
        }
        let refused = request
            .advanced()
            .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                original.as_ref().unwrap(),
                RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
                WorthQueryOutputDemandControls::default(),
            );
        assert!(
            matches!(&refused, Err(Preparation::Demand(Demand::Demand(cause)))
            if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
            "live source is superseded: {:?}",
            refused.err()
        );
    }
    drop(home);
    let home = open_ordinary(&directory, false, execution);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(input(first))
        .without_source()
        .idempotency(&1_u64)
        .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
    ) = outcome
    else {
        panic!("the superseded source still has its original keyed receipt");
    };
    let refused = request
        .advanced()
        .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            host,
            &original,
            RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
            WorthQueryOutputDemandControls::default(),
        );
    assert!(
        matches!(&refused, Err(Preparation::Demand(Demand::Demand(cause)))
        if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
        "reopen cannot revive retired custody: {:?}",
        refused.err()
    );
    let rows = request
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        rows.rows()[0].retention_days,
        second,
        "original-key replay cannot repeat the old source edit"
    );
}
