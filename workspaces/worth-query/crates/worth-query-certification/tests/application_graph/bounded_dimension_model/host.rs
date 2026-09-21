//! Publishing a bounded-dimension host through the real installation path.
//!
//! Every host in this court is built here and nowhere else: one installed
//! schema carrying both rule contracts, one explicit program roster, and one
//! seeded part. Which program a host starts on is the only thing that varies,
//! which is what makes a difference in behaviour attributable to it.

use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
};
use worth_query_host::facade::application_installation::{
    in_memory_rostered_program, WorthQueryApplicationProgramRoster,
    WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
    WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
};
use worth_query_host::facade::{declaration, primary_graph, runtime};

use super::assessment_output::{
    PartAssessmentBinding, PartAssessmentHandler, PartAssessmentProducer, PartAssessmentProvider,
};
use super::assessment_readiness::PartAssessmentReadiness;
use super::dimension_entry::{SetPartDimensionBinding, SetPartDimensionHandler, PART_IDENTITY};
use super::programs::{
    validated_changed_feature_program, validated_changed_operation_program,
    validated_first_program, validated_first_resource_program, validated_foreign_rule_program,
    validated_removed_operation_program, validated_second_program,
    validated_second_resource_program, DimensionProgramP0, DimensionProgramP1,
    ResourceDimensionProgramP0,
};
use super::rules::{resolve_first_rule, resolve_second_rule};
use super::schema::{
    BoundedDimensionContribution, BoundedDimensionSchema, BoundedDimensionV1, BoundedDimensionV2,
    Part, PartDimensionField, PartIdentityField, PartPrincipalBinding,
};
use super::workflow::{
    WorkflowAdvanceBinding, WorkflowAdvanceHandler, WorkflowApprovalBinding,
    WorkflowApprovalHandler, WorkflowDefinitionAuthoringBinding,
    WorkflowDefinitionAuthoringHandler, WorkflowInstanceStartBinding, WorkflowInstanceStartHandler,
};

/// The dimension every host seeds. It satisfies both installed rules, so the
/// same bootstrap is lawful whichever program the host starts on.
pub const SEED_DIMENSION: u64 = 7;

pub type BoundedDimensionRuntime<Initial> =
    WorthQueryProgramApplicationRuntime<BoundedDimensionSchema, Initial>;
pub type BoundedDimensionWorkflowRuntime = WorthQueryWorkflowApplicationRuntime<
    BoundedDimensionSchema,
    super::workflow::ReviewedGeometryWorkflow,
    DimensionProgramP0,
>;

impl WorthQueryApplicationContribution<BoundedDimensionSchema> for BoundedDimensionContribution {
    type Configuration = ();

    fn contracts(
        contracts: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionContracts<BoundedDimensionSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<PartAssessmentProducer>()?;
        contracts.conditional::<PartAssessmentReadiness>()?;
        Ok(())
    }

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, BoundedDimensionSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            BoundedDimensionV1::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            resolve_first_rule,
        )?;
        setup.invariant(
            BoundedDimensionV2::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            resolve_second_rule,
        )?;
        setup
            .handler::<SetPartDimensionBinding, _>(SetPartDimensionHandler)
            .and_then(|()| setup.handler::<PartAssessmentBinding, _>(PartAssessmentHandler))
            .and_then(|()| setup.producer::<PartAssessmentProducer>(PartAssessmentProvider))
            .and_then(|()| setup.conditional::<PartAssessmentReadiness>(()))
            .and_then(|()| {
                setup.handler::<WorkflowDefinitionAuthoringBinding, _>(
                    WorkflowDefinitionAuthoringHandler,
                )
            })
            .and_then(|()| {
                setup.handler::<WorkflowInstanceStartBinding, _>(WorkflowInstanceStartHandler)
            })
            .and_then(|()| setup.handler::<WorkflowAdvanceBinding, _>(WorkflowAdvanceHandler))
            .and_then(|()| setup.handler::<WorkflowApprovalBinding, _>(WorkflowApprovalHandler))
    }
}

/// Publishes a host whose first occurrence runs P0, with P1 rostered beside it.
pub fn publish_on_first_program() -> BoundedDimensionRuntime<DimensionProgramP0> {
    publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_operation_program()),
    )
    .expect("the P0-initial bounded-dimension host must install")
}

pub fn publish_workflow_on_first_program() -> BoundedDimensionWorkflowRuntime {
    super::workflow::retain_workflow(publish_on_first_program())
}

/// Publishes a host whose first occurrence runs P1, with P0 rostered beside it.
pub fn publish_on_second_program() -> BoundedDimensionRuntime<DimensionProgramP1> {
    publish(
        validated_second_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_first_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_operation_program()),
    )
    .expect("the P1-initial bounded-dimension host must install")
}

pub fn publish_on_first_resource_program() -> BoundedDimensionRuntime<ResourceDimensionProgramP0> {
    publish(
        validated_first_resource_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_resource_program())
            .support(validated_second_program()),
    )
    .expect("the resource-custody host must install")
}

/// Attempts a host that rosters P0 alone against the two-rule catalog.
pub fn publish_first_program_alone(
) -> Result<BoundedDimensionRuntime<DimensionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new(),
    )
}

/// Attempts a host that rosters a program declaring an uninstalled rule.
pub fn publish_with_foreign_rule_rostered(
) -> Result<BoundedDimensionRuntime<DimensionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_operation_program())
            .support(validated_foreign_rule_program()),
    )
}

fn publish<Initial>(
    initial: ValidatedApplicationProgram<BoundedDimensionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, BoundedDimensionSchema>,
) -> Result<BoundedDimensionRuntime<Initial>, WorthQueryInMemoryApplicationDenial>
where
    Initial: ApplicationProgramDefinition<
            BoundedDimensionSchema,
            Contributions = (BoundedDimensionContribution,),
        > + 'static,
    Initial::Outputs:
        ApplicationProgramOutputsShape<BoundedDimensionSchema>
            + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
                BoundedDimensionSchema,
            >,
{
    in_memory_rostered_program(
        initial,
        roster,
        BoundedDimensionSchema::declaration().expect("the bounded-dimension schema is valid"),
        ((),),
        host_limits(),
        |graph, installed| {
            let principal_binding = installed
                .principal_binding(PartPrincipalBinding::reference())
                .expect("the part principal binding must install");
            seed_operator(graph, &principal_binding);
            seed_part(graph);
            super::workflow::seed_authoring(graph);
            Ok(())
        },
    )
}

fn seed_operator(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<BoundedDimensionSchema>,
    principal_binding: &worth_query_host::facade::domain::WorthQueryInstalledPrincipalBinding<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        super::schema::ExternalMapping,
        super::schema::Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
    >,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("bounded-dimension-operator")
                .expect("the principal key is valid"),
            1_u64,
            super::operator_identity::operator_identity(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("the operator must seed");
}

fn seed_part(graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<BoundedDimensionSchema>) {
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                Part::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("part-row-1")
                    .expect("the row key is valid"),
            )
            .field(PartIdentityField::reference(), PART_IDENTITY.to_owned())
            .field(PartDimensionField::reference(), SEED_DIMENSION),
        )
        .expect("the part must seed");
}

fn host_limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            4_096,
            2 * 1024 * 1024,
            1_048_576,
        )
        .expect("valid candidate limits"),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .expect("valid query limits"),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn world_resources() -> runtime::WorthQueryProductWorldResources {
    runtime::WorthQueryProductWorldResources::install(
        runtime::RuntimeWorldBudgetInstallation {
            branches: runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 256,
                history_metadata_bytes: 4 * 1024 * 1024,
            },
            observations: runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: 128,
            },
            publication: runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 256,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        runtime::WorthQueryProductWorldClock::start(),
    )
    .expect("the bounded-dimension World resources are valid")
}
