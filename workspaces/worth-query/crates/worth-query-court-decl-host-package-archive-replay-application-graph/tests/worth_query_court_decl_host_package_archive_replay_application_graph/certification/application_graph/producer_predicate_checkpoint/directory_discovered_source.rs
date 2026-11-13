//! Abandoned discovered source retirement persists; live handles still protect it.
use super::ordinary_source::*;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationDiscoveredMutationOutcome, WorthQueryApplicationMutationOutcome,
        WorthQueryRequiredOutputPreparationDenial as Preparation,
    },
    application_installation::ApplicationBuildIdentity,
    primary_graph::{
        WorthQueryApplicationDiscoveredOutputConnection, WorthQueryOutputDemandDenialKind,
    },
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    schema::document_retention_query::DocumentRetentionRow,
};

#[test]
fn directory_discovered_source_retirement_preserves_live_handles_and_survives_reopen() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = |initialize: bool| {
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            ApplicationProgramAuthoring::<DocumentRetentionSchema, Program>::begin().validated_program().unwrap(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program())
                .support(validated_actor_program()).support(validated_model_lifecycle_program()),
            DocumentRetentionSchema::declaration().unwrap(), ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            move |phase, graph, schema| { assert!(initialize); seed_host(phase, graph, schema) },
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(request.open_in_advancement(&phase).unwrap())
        })
        .unwrap()
    };
    let input = |days| SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: days,
        },
    };
    // Caller-chosen keys/values; none is the seeded value seven.
    let home = open(true);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        for days in [4_u64, 5] {
            let outcome = request
                .advanced()
                .mutate(input(days))
                .without_source()
                .idempotency(&days)
                .execute_performed_discovered::<Program, Root>(
                    host,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let WorthQueryApplicationDiscoveredMutationOutcome::Performed(done) = outcome else {
                panic!("actual discovered source must perform");
            };
            if days == 5 {
                // Keep B live while C is performed: the owner must not retire B.
                let newer=request.advanced().mutate(input(6)).without_source().idempotency(&6_u64)
                    .execute_performed_discovered::<Program,Root>(host, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
                assert!(matches!(
                    newer,
                    WorthQueryApplicationDiscoveredMutationOutcome::Performed(_)
                ));
                let _still_live = request
                    .advanced()
                    .recover_discovered_required_outputs::<Program, Root>(
                        host,
                        done.receipt(),
                        WorthQueryOutputDemandControls::default(),
                    )
                    .expect("B is protected by its live handle");
                break;
            }
        }
    }
    drop(home);
    let home = open(false);

    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let replay = request
            .advanced()
            .mutate(input(4))
            .without_source()
            .idempotency(&4_u64)
            .execute_performed_discovered::<Program, Root>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
        ) = replay
        else {
            panic!("A's keyed effect still exists");
        };
        let refused = request
            .advanced()
            .recover_discovered_required_outputs::<Program, Root>(
                host,
                &original,
                WorthQueryOutputDemandControls::default(),
            );
        assert!(
            matches!(&refused,Err(Preparation::DemandExecution(cause)) if cause.kind()==WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
            "A cannot resurrect: {:?}",
            refused.err()
        );
        // C has not been locally reissued. A new source must retire it durably too.
        let outcome = request
            .advanced()
            .mutate(input(8))
            .without_source()
            .idempotency(&8_u64)
            .execute_performed_discovered::<Program, Root>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationDiscoveredMutationOutcome::Performed(_)
        ));
        assert_eq!(
            host.runtime().producer_contacts_on_this_thread_for_test(),
            1
        );
    }
    drop(home);
    let home = open(false);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    for days in [6_u64, 8] {
        let replay = request
            .advanced()
            .mutate(input(days))
            .without_source()
            .idempotency(&days)
            .execute_performed_discovered::<Program, Root>(
                host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(original),
        ) = replay
        else {
            panic!("each key returns its original effect");
        };
        let recovered = request
            .advanced()
            .recover_discovered_required_outputs::<Program, Root>(
                host,
                &original,
                WorthQueryOutputDemandControls::default(),
            );
        if days == 6 {
            assert!(
                matches!(&recovered,Err(Preparation::DemandExecution(cause)) if cause.kind()==WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
                "cold C cannot resurrect: {:?}",
                recovered.err()
            );
        } else {
            let mut recovered = recovered.expect("D re-enters its recorded discovery recipe");
            assert!(matches!(recovered.settle(host,&request).unwrap(),
                worth_query_host::facade::application_entry::WorthQueryDiscoveredProgramOutputProgress::Settled(_)));
        }
    }
    assert_eq!(
        host.runtime().producer_contacts_on_this_thread_for_test(),
        0,
        "reopened D uses its original completion"
    );
    assert_eq!(
        request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        8
    );
}

pub struct Connection;
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "discovered-retention.connection";
}
impl
    ApplicationOccurrenceConnectionBinding<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        AssessmentFeature,
    > for Connection
{
}
impl WorthQueryApplicationDiscoveredOutputConnection<DocumentRetentionSchema> for Connection {
    type Source = SetRetentionBinding;
    type Discovery = DocumentRetentionRead;
    type Demand = RetentionAssessmentDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_source(
        input: &SetRetentionInput,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(DocumentRetentionRead {
            identity: input.identity.clone(),
        })
    }
    fn demands_from_discovery(
        row: &DocumentRetentionRow,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![RetentionAssessmentDemand::new(&row.identity)])
    }
}
pub type Root = ApplicationDiscoveredOutputGraph<
    ApplicationConnectionRef<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        RetentionOutput,
        AssessmentFeature,
        RetentionInput,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
pub struct Delivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, SetRetentionBinding> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "discovered-retention.delivery";
}
impl ApplicationDiscoveredActionDelivery<DocumentRetentionSchema, SetRetentionBinding>
    for Delivery
{
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for Program {
    type Contributions =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("discovered-retention.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .provides::<RetentionOutput>()
                .action::<SetRetentionBinding, Delivery>()
                .finish_action()
                .action::<ReviewedSetRetentionBinding, RetentionCompletion>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, AssessmentFeature>()
                .conditional_operation::<PublishRetentionAssessment>()
                .finish(),
        ]
    }
}
