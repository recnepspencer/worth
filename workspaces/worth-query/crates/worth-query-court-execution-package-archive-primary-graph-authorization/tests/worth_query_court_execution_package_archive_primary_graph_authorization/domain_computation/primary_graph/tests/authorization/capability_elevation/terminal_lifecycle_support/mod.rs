pub use worth_execution::ExecutionAllocationPolicy;
pub use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
pub use worth_query_declaration::facade::application_schema::OperationReads;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityElevation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityElevationClosedAt;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityReview;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityReviewReviewedAt;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationApprover;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationGrant;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationNotAfter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationNotBefore;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationReason;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationRequester;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationResource;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationReview;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewKindField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewResource;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewer;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::close::CloseElevationInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CompleteCapabilityReviewOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CompleteElevationReviewCapability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::review::CompleteElevationReviewInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RevokeCapabilityElevationOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RevokeElevationCapability;
pub use worth_query_execution::facade::domain_computation::WorthQueryApprovedElevation;
pub use worth_query_execution::facade::domain_computation::WorthQueryCompleteApplicationReadSet;
pub use worth_query_execution::facade::domain_computation::WorthQueryElevationCloseOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryElevationCloseProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryMandatoryReview;
pub use worth_query_execution::facade::domain_computation::WorthQueryMandatoryReviewOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryMandatoryReviewProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryProjectedApplicationMutation;
pub use worth_query_execution::facade::domain_computation::WorthQueryReviewedElevation;

pub use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::Authenticated;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::World;

pub type ElevationIdentity =
    WorthQueryInvariantEntityIdentity<IdentityExecutionSchema, CapabilityElevation>;
pub type ReviewIdentity =
    WorthQueryInvariantEntityIdentity<IdentityExecutionSchema, CapabilityReview>;

pub type CloseReads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;
pub type ReviewReads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

pub fn close_access(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        RevokeElevationCapability,
        RevokeCapabilityElevationOperation,
        CloseElevationInput,
    >,
    WorthQueryOperationAuthorizationDenial,
>{
    close_access_with_input(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        close_input(),
    )
}

pub fn close_access_with_input(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: CloseElevationInput,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        RevokeElevationCapability,
        RevokeCapabilityElevationOperation,
        CloseElevationInput,
    >,
    WorthQueryOperationAuthorizationDenial,
>{
    let capability = world
        .application
        .installed_schema()
        .capability(
            RevokeElevationCapability::reference(),
            RevokeCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub fn review_access(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        CompleteElevationReviewCapability,
        CompleteCapabilityReviewOperation,
        CompleteElevationReviewInput,
    >,
    WorthQueryOperationAuthorizationDenial,
>{
    review_access_with_input(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        review_input(),
    )
}

pub fn review_access_with_input(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: CompleteElevationReviewInput,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        CompleteElevationReviewCapability,
        CompleteCapabilityReviewOperation,
        CompleteElevationReviewInput,
    >,
    WorthQueryOperationAuthorizationDenial,
>{
    let capability = world
        .application
        .installed_schema()
        .capability(
            CompleteElevationReviewCapability::reference(),
            CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub fn materialize_close(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryElevationCloseProgram<
    IdentityExecutionSchema,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
    Account,
> {
    close_reads(
        &(phase_custody).test_host_request(),
        world,
        request,
        approved,
    )
    .materialize_elevation_close_program()
    .unwrap()
}

pub(super) use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_reads::close_reads;

pub fn close_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryMandatoryReview {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            materialize_close(
                &(phase_custody).test_host_request(),
                world,
                request,
                approved,
            )
        })
        .expect("the lifecycle materialization owns its host call");
    match world
        .application
        .compare_and_commit_elevation_close(program, idempotency(173, 173))
    {
        WorthQueryElevationCloseOutcome::Closed(mandatory) => mandatory,
        _ => panic!("the canonical close transition must commit"),
    }
}

pub fn materialize_review(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryMandatoryReviewProgram<
    IdentityExecutionSchema,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
    Account,
> {
    review_reads(
        &(phase_custody).test_host_request(),
        world,
        request,
        mandatory,
    )
    .materialize_mandatory_review_program()
    .unwrap()
}

pub fn review_reads(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> ReviewReads {
    let reviewer =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(phase_custody, world, "carol", request);
    let access = review_access(phase_custody, world, &reviewer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CompleteCapabilityReviewOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_mandatory_review(
            &phase_custody.phase_for_application(&world.application),
            mandatory,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application(&world.application),
            &admission,
            |reader, _| {
                seal_lifecycle_facts(reader);
                let review = reader
                    .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
                    .unwrap();
                reader
                    .require_decision_field(&review, CapabilityReviewReviewedAt::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

pub fn review_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryReviewedElevation {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            materialize_review(&phase_custody, world, request, mandatory)
        })
        .expect("the lifecycle materialization owns its host call");
    match world
        .application
        .compare_and_commit_mandatory_review(program, idempotency(174, 174))
    {
        WorthQueryMandatoryReviewOutcome::Reviewed(reviewed) => reviewed,
        _ => panic!("the canonical mandatory review must commit"),
    }
}

pub fn seal_lifecycle_facts<Operation>(
    reader: &mut worth_query_execution::facade::domain_computation::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
) where
    CapabilityElevationIdentity: OperationReads<Operation>,
    CapabilityElevationReason: OperationReads<Operation>,
    CapabilityElevationStatusField: OperationReads<Operation>,
    CapabilityElevationNotBefore: OperationReads<Operation>,
    CapabilityElevationNotAfter: OperationReads<Operation>,
    CapabilityReviewIdentity: OperationReads<Operation>,
    CapabilityReviewKindField: OperationReads<Operation>,
    CapabilityReviewStatusField: OperationReads<Operation>,
    CapabilityElevationRequester: OperationReads<Operation>,
    CapabilityElevationApprover: OperationReads<Operation>,
    CapabilityElevationGrant: OperationReads<Operation>,
    CapabilityElevationResource: OperationReads<Operation>,
    CapabilityElevationReview: OperationReads<Operation>,
    CapabilityReviewResource: OperationReads<Operation>,
    CapabilityReviewer: OperationReads<Operation>,
{
    let elevation = reader
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
        )
        .unwrap();
    let review = reader
        .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
        .unwrap();
    seal_lifecycle_fields(reader, &elevation, &review);
    seal_lifecycle_relations(reader, &elevation, &review);
}

pub fn close_input() -> CloseElevationInput {
    CloseElevationInput {
        account: "account-1".to_owned(),
        elevation: "elevation-2".to_owned(),
    }
}

pub fn review_input() -> CompleteElevationReviewInput {
    CompleteElevationReviewInput {
        account: "account-1".to_owned(),
        elevation: "elevation-2".to_owned(),
        review: "review-2".to_owned(),
    }
}

pub use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::lifecycle_facts::seal_lifecycle_fields;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::lifecycle_facts::seal_lifecycle_relations;

#[path = "close_reads/mod.rs"]
pub mod close_reads;
#[path = "lifecycle_facts/mod.rs"]
pub mod lifecycle_facts;
