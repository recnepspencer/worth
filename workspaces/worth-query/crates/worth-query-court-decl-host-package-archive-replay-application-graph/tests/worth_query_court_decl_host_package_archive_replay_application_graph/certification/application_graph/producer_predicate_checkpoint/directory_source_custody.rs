//! Required source custody at source-only, queued and root-settled teardown cuts.
//! The recovery call below deliberately selects Recorded, never an old live receipt.
use super::ordinary_source::*;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::ApplicationBuildIdentity,
    primary_graph::WorthQueryOutputSourceReplay,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
};

#[test]
fn directory_required_source_and_output_resume_without_repeating_the_source_effect() {
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationOutputDemandDenial as Demand,
        WorthQueryRequiredOutputPreparationDenial as Preparation,
    };
    use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = |initialize| open_ordinary(&directory, initialize, execution);
    let days = [1,2].into_iter().find(|days| *days !=
        worth_query_testkit_document_retention::facade::document_retention_model::host::SEED_RETENTION).unwrap();
    let key = 1_u64; // Caller-chosen input; issued Native and World IDs are compared below.
    let input = || SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: days,
        },
    };

    let home = open(true);
    let (source_head, native_source) = {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(input())
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
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            1
        );
        let receipt = performed.receipt();
        let commit = receipt.committed_product_publication().composite_commit();
        (
            (commit.owner_identity().durable().unwrap(), commit.ordinal()),
            receipt.commit_reference().clone(),
        )
    };
    drop(home); // Source-only cut. No live source or request crosses it.

    let home = open(false);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let replay = request
            .advanced()
            .mutate(input())
            .without_source()
            .idempotency(&key)
            .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
        ) = replay
        else {
            panic!("source-only reopen must recover the original keyed occurrence");
        };
        let recorded = original
            .new_product_head()
            .durable()
            .unwrap()
            .selected_commit();
        assert_eq!((recorded.owner(), recorded.ordinal()), source_head);
        assert_eq!(original.commit_reference(), &native_source);
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            0
        );
        let _queued = request
            .advanced()
            .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                WorthQueryOutputSourceReplay::Recorded(&original),
                RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap();
    }
    drop(home); // Queued cut, on that same original source and demand.

    let home = open(false);
    let original_output = {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let replay = request
            .advanced()
            .mutate(input())
            .without_source()
            .idempotency(&key)
            .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
        ) = replay
        else {
            panic!("queued reopen must retain the same source");
        };
        let mut outputs = request
            .advanced()
            .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                &original,
                RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap();
        while outputs.settled_root_observation().is_none() {
            assert!(
                !matches!(
                    outputs.advance(host, &request).unwrap(),
                    WorthQueryApplicationProgramOutputProgress::Settled(_)
                ),
                "root settlement yields before source completion"
            );
        }
        let mut read = request
            .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .start_in_program::<_, OrdinaryRoot>(host)
            .unwrap();
        let worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress::Settled(settled) = read.settle(&request).unwrap()
            else { panic!("root output is readable before source completion"); };
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            0
        );
        settled
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id()
    };
    drop(home); // Root-settled cut, before source/tree completion.

    let home = open(false);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let replay = request
            .advanced()
            .mutate(input())
            .without_source()
            .idempotency(&key)
            .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
        ) = replay
        else {
            panic!("root-settled reopen must retain the same source");
        };
        assert_eq!(original.commit_reference(), &native_source);
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            0
        );
        let mut outputs = request
            .advanced()
            .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
                host,
                &original,
                RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap();
        let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
            outputs.settle(host, &request).unwrap()
        else {
            panic!("the original output obligation must complete");
        };
        assert_eq!(
            settled
                .root_settlement()
                .outputs_of::<RetentionAssessmentOutputs>()
                .unwrap()
                .entity::<AssessmentOutput>()
                .unwrap()
                .entity_id(),
            original_output
        );
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            0,
            "all recovered stages reuse the original producer completion"
        );
        let rows = request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(rows.rows()[0].retention_days, days);
    }
    drop(home);
    let home = open(false);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let replay = request
        .advanced()
        .mutate(input())
        .without_source()
        .idempotency(&key)
        .execute_performed::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
    ) = replay
    else {
        panic!("terminal replay must keep the original source effect");
    };
    assert_eq!(original.commit_reference(), &native_source);
    let refused = request
        .advanced()
        .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            host,
            &original,
            RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
            WorthQueryOutputDemandControls::default(),
        );
    assert!(
        matches!(&refused,Err(Preparation::Demand(Demand::Demand(cause)))
        if cause.kind()==WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
        "terminal custody must remain discharged: {:?}",
        refused.err()
    );
    assert_eq!(
        host.runtime().producer_contacts_on_this_thread_for_test(),
        0
    );
}

pub(super) fn open_ordinary(
    directory: &std::path::Path,
    initialize: bool,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Box<
    worth_query_host::facade::WorthQueryApplicationHome<
        DocumentRetentionSchema,
        OrdinaryAssessmentProgram,
    >,
> {
    let program =
        ApplicationProgramAuthoring::<DocumentRetentionSchema, OrdinaryAssessmentProgram>::begin()
            .validated_program()
            .unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(directory.to_path_buf()),
                program,
                WorthQueryApplicationProgramRoster::new()
                    .support(validated_second_program())
                    .support(validated_actor_program())
                    .support(validated_model_lifecycle_program()),
                DocumentRetentionSchema::declaration().unwrap(), ((),),
                ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
                host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
                move |phase, graph, schema| {
                    assert!(initialize, "reopen must not initialize the source");
                    seed_host(phase, graph, schema)
                },
            );
    with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("required-source Home: {cause:?}")),
        )
    })
    .unwrap()
}
