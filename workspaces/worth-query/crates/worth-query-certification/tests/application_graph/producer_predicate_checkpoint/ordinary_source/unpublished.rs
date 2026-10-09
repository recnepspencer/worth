//! A real NoSource required-root append fault, recovered without authoring again.
use super::*;
use crate::document_retention_model::retention_entry::{
    candidate_count, decision_count, reset_candidate_count, reset_decision_count,
};
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
    },
    application_entry::{
        WorthQueryApplicationRecoveryRequestDenial, WorthQueryDiscoveredRecoveryProgress,
    },
    primary_graph::WorthQueryApplicationIdempotencyResolution,
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};
// The real v2 retention rule admits the observation value 19. The earlier
// ordinary-source court deliberately retains its original v1 rule (maximum 10).
struct RequiredRecoveryProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for RequiredRecoveryProgram {
    type Contributions = <OrdinaryAssessmentProgram as ApplicationProgramDefinition<
        DocumentRetentionSchema,
    >>::Contributions;
    type Outputs = ApplicationProgramOutputs<OrdinaryRoot>;
    type Rules =
        <crate::document_retention_model::programs::RetentionProgramP1 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.ordinary-assessment-recovery.program.v1",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <OrdinaryAssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::feature_specs()
    }
}

#[test]
fn initial_required_partial_recovers_original_demand_without_handler_reexecution() {
    // Reserved only for ordinary handler observations; operation semantics are unchanged.
    const RETENTION: u64 = 19;
    reset_candidate_count(RETENTION);
    reset_decision_count(RETENTION);
    let program =
        ApplicationProgramAuthoring::<DocumentRetentionSchema, RequiredRecoveryProgram>::begin()
            .validated_program()
            .unwrap();
    let host = publish(
        program,
        WorthQueryApplicationProgramRoster::new()
            .support(crate::document_retention_model::programs::validated_first_program()),
    )
    .unwrap();
    let intent = SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: RETENTION,
        },
    };
    let key = 0xaced_1901;
    // The original request, scope and principal die before the pending owner advances.
    let mut recovery = {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        host.fail_next_durable_append_for_test();
        match host
            .request(&principal, &scope)
            .mutate(intent.clone())
            .without_source()
            .idempotency(&key)
            .execute_performed::<RequiredRecoveryProgram, OrdinaryRoot>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
        {
            WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(recovery) => recovery,
            WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
                panic!("actual append fault did not reach a required native partial: {outcome:?}")
            }
            WorthQueryApplicationPerformedMutationOutcome::Blocked(blocked) => {
                panic!(
                    "actual append fault retained a different native posture: {:?}",
                    blocked.outcome()
                )
            }
            WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => {
                panic!(
                    "actual append fault unexpectedly published: {:?}",
                    performed.receipt()
                )
            }
            WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
                panic!(
                    "actual append fault published but refused required custody: {:?}; {:?}",
                    failure.receipt(),
                    failure.denial(),
                )
            }
        }
    };
    assert_eq!(
        recovery.initial_cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert_eq!(
        (decision_count(RETENTION), candidate_count(RETENTION)),
        (1, 1)
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    cancellation.cancel();
    assert!(matches!(
        host.request(&principal, &stopped)
            .mutate(intent.clone())
            .without_source()
            .idempotency(&key)
            .recover_unpublished_required_in_program(&mut recovery, &host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Interrupted(
            WorthQueryRequestInterruption::Cancelled
        ))
    ));
    assert!(recovery.performed().is_none());
    assert_eq!(
        host.request(&principal, &scope)
            .mutate(intent.clone())
            .without_source()
            .idempotency(&key)
            .recover_unpublished_required_in_program(&mut recovery, &host)
            .unwrap(),
        WorthQueryDiscoveredRecoveryProgress::Performed
    );
    let (source, initial, performed, prior) = host
        .request(&principal, &scope)
        .mutate(intent)
        .without_source()
        .idempotency(&key)
        .promote_recovered_required_outputs(recovery, &host)
        .unwrap_or_else(|(denial, _)| panic!("native promotion: {denial:?}"))
        .into_parts();
    assert_eq!(initial, ProductUnpublishedCause::SettlementPending);
    assert!(prior.is_empty());
    let (read, publication, cleanup) = performed.into_parts();
    assert!(publication.is_none());
    assert!(cleanup.is_none());
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) =
        read.unwrap().into_resolution()
    else {
        panic!("the exact original key must be published");
    };
    assert_eq!(
        source.receipt().commit_reference(),
        receipt.commit_reference()
    );
    let controls = WorthQueryOutputDemandControls::new(
        std::num::NonZeroUsize::new(4096).unwrap(),
        std::num::NonZeroUsize::new(8192).unwrap(),
    );
    let request = host.request(&principal, &scope);
    let mut outputs = source
        .start_required_outputs(&host, &request, controls)
        .unwrap_or_else(|(_, denial)| panic!("original fixed demand: {denial:?}"));
    assert_eq!(
        outputs.source_observation().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
        outputs.settle(&host, &request).unwrap()
    else {
        panic!("the real assessment producer must settle");
    };
    assert_eq!(
        request
            .at(settled.observation())
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        RETENTION
    );
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    assert!(settled
        .root_receipt()
        .unwrap()
        .outputs_of::<RetentionAssessmentOutputs>()
        .unwrap()
        .entity::<AssessmentOutput>()
        .is_ok());
    assert_eq!(
        (decision_count(RETENTION), candidate_count(RETENTION)),
        (1, 1)
    );
    assert_eq!(host.retained_source_custody_count_for_test(), 0);
}
