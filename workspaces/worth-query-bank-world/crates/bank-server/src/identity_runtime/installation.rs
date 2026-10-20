use bank_domain::{
    queries::{PaymentAmountQueryBinding, PaymentDetailQueryBinding},
    schema::{
        ApprovePayment, ApprovePaymentMutationBinding, ApprovedBusinessPaymentAdvance,
        ApprovedBusinessPaymentAdvanceOperation, ApprovedBusinessPaymentApproval,
        ApprovedBusinessPaymentApprovalOperation, ApprovedBusinessPaymentAuthoring,
        ApprovedBusinessPaymentAuthoringBinding, ApprovedBusinessPaymentAuthoringOperation,
        ApprovedBusinessPaymentInstanceStart, ApprovedBusinessPaymentInstanceStartOperation,
        ApprovedBusinessPaymentWorkflow, BankPrincipalBinding, BankSchema,
    },
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_host::facade::{
    application_installation::{
        in_memory_rostered_program, in_memory_rostered_program_with_authorization_time_source,
        WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationLimits,
    },
    domain::{
        WorthQueryApplicationWorkflowResourceCeiling,
        WorthQueryApplicationWorkflowSpecInstallation, WorthQueryInstalledApplicationSchema,
    },
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
        WorthQueryRuntimeTimeSource,
    },
    runtime::WorthQueryApplicationQueryResourceProfile,
};

use super::{BankGraphSeed, BankIdentityRuntime, InstalledBankPrincipalBinding};
use crate::{
    application_definition::{
        validated_bank_application, validated_bank_application_p1, validated_bank_application_p2,
    },
    approval_authentication::install_approval_authentication,
    error::BankIdentityRuntimeBuildError,
    graph_bootstrap::bind_bank_world_with_estate,
    principal_seed::PreparedBankPrincipalSeed,
    BankApprovalAuthenticationConfiguration,
};

pub(super) enum BankAuthorizationTimeInstallation {
    System,
    Installed(Box<dyn WorthQueryRuntimeTimeSource>),
}

pub(super) fn install_prepared(
    seeds: Vec<PreparedBankPrincipalSeed>,
    world: Option<BankGraphSeed>,
    authorization_time: BankAuthorizationTimeInstallation,
    approval_authentication: BankApprovalAuthenticationConfiguration,
) -> Result<BankIdentityRuntime, BankIdentityRuntimeBuildError> {
    prepare_installation(seeds, world, authorization_time, approval_authentication)?.bind()
}

pub(super) fn prepare_installation(
    seeds: Vec<PreparedBankPrincipalSeed>,
    world: Option<BankGraphSeed>,
    authorization_time: BankAuthorizationTimeInstallation,
    approval_authentication: BankApprovalAuthenticationConfiguration,
) -> Result<super::BankIdentityRuntimeWorkflowBinding, BankIdentityRuntimeBuildError> {
    let program = validated_bank_application()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let successor = validated_bank_application_p1()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let successor_p2 = validated_bank_application_p2()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let deployment_revisions = [
        *program.revision(),
        *successor.revision(),
        *successor_p2.revision(),
    ];
    let declaration =
        BankSchema::declaration().map_err(BankIdentityRuntimeBuildError::SchemaDeclaration)?;
    let limits = bank_application_limits();
    let mut invariant_projection = None;
    let initialize =
        |_phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>,
         graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
         installed: &WorthQueryInstalledApplicationSchema<BankSchema>| {
            bind_principals(graph, installed, seeds)?;
            crate::graph_bootstrap::bind_program_deployment_scopes(graph, &deployment_revisions)?;
            bind_world_truth(graph, world.as_ref())?;
            invariant_projection = Some(graph.retain_invariant_projection_authority());
            Ok(())
        };
    let runtime = match authorization_time {
        BankAuthorizationTimeInstallation::System => in_memory_rostered_program(
            program,
            WorthQueryApplicationProgramRoster::new()
                .support(successor)
                .support(successor_p2),
            declaration,
            ((), (), ()),
            limits,
            initialize,
        ),
        BankAuthorizationTimeInstallation::Installed(source) => {
            in_memory_rostered_program_with_authorization_time_source(
                program,
                WorthQueryApplicationProgramRoster::new()
                    .support(successor)
                    .support(successor_p2),
                declaration,
                ((), (), ()),
                limits,
                initialize,
                source,
            )
        }
    }
    .map_err(|denial| BankIdentityRuntimeBuildError::ApplicationInstallation(Box::new(denial)))?;
    let invariant_projection = invariant_projection
        .expect("program construction invokes the initializer before publication");
    let binding = resolve_installed_principal_binding(runtime.runtime())?;
    let approval_authentication = install_approval_authentication(
        runtime.runtime().installed_schema(),
        approval_authentication,
    )
    .map_err(BankIdentityRuntimeBuildError::WorkflowAuthenticationInstallation)?;
    let workflow = install_payment_workflow_spec(
        runtime.runtime().installed_schema(),
        runtime.installed_program(),
    )
    .map_err(BankIdentityRuntimeBuildError::WorkflowInstallation)?;
    Ok(super::BankIdentityRuntimeWorkflowBinding::new(
        runtime,
        workflow,
        binding,
        invariant_projection,
        approval_authentication,
    ))
}

pub(super) fn install_payment_workflow_spec<Program>(
    schema: &WorthQueryInstalledApplicationSchema<BankSchema>,
    program: &worth_query_host::facade::domain::WorthQueryInstalledApplicationProgram<
        BankSchema,
        Program,
    >,
) -> Result<
    worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
    >,
    worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenial,
>
where
    Program:
        worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            BankSchema,
        >,
{
    WorthQueryApplicationWorkflowSpecInstallation::<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        Program,
    >::begin(
        schema,
        program,
        payment_workflow_resources(),
    )
    .operation::<ApprovedBusinessPaymentAuthoringBinding>()?
    .operation::<ApprovePaymentMutationBinding>()?
    .assessment::<PaymentDetailQueryBinding>()?
    .condition_operand::<PaymentAmountQueryBinding>()?
    .approval::<
        ApprovedBusinessPaymentApproval,
        ApprovedBusinessPaymentApprovalOperation,
        ApprovePayment,
    >()?
    .authoring_capability::<
        ApprovedBusinessPaymentAuthoring,
        ApprovedBusinessPaymentAuthoringOperation,
        ApprovePayment,
    >()?
    .instance_start_capability::<
        ApprovedBusinessPaymentInstanceStart,
        ApprovedBusinessPaymentInstanceStartOperation,
        ApprovePayment,
    >()?
    .advance_capability::<
        ApprovedBusinessPaymentAdvance,
        ApprovedBusinessPaymentAdvanceOperation,
        ApprovePayment,
    >()?
    .finish()
}

fn payment_workflow_resources() -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1_024,
        32,
        128,
        256 * 1_024,
    )
    .expect("approved-payment workflow resources are nonzero")
}

pub(crate) fn bank_application_limits() -> WorthQueryInMemoryApplicationLimits {
    let queries = WorthQueryApplicationQueryResourceProfile::bounded(32_768, 262_144, 32_768, 64)
        .expect("bank application-query resource profile is statically non-zero");
    WorthQueryInMemoryApplicationLimits::new(
        super::product_world_resources::bank_product_world_resources(),
        queries,
        SignalConditionalEvaluationBudget::development(),
    )
}

fn bind_principals(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    installed: &WorthQueryInstalledApplicationSchema<BankSchema>,
    seeds: Vec<PreparedBankPrincipalSeed>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(BankPrincipalBinding::reference())
        .map_err(|error| {
            WorthQueryPrimaryGraphInstallationDenial::binding_not_installed(error.to_string())
        })?;
    for seed in seeds {
        graph.bind_principal(
            &binding,
            seed.key,
            seed.principal_id,
            seed.external_identity,
            seed.status,
        )?;
    }
    Ok(())
}

fn bind_world_truth(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: Option<&BankGraphSeed>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let Some(world) = world else {
        return Ok(());
    };
    bind_bank_world_with_estate(
        graph,
        &world.snapshot,
        &world.owners,
        &world.employees,
        world.estate.as_ref(),
    )?;
    crate::graph_bootstrap::bind_program_deployment_grants(graph, &world.deployment_grants)
}

fn resolve_installed_principal_binding(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
) -> Result<InstalledBankPrincipalBinding, BankIdentityRuntimeBuildError> {
    runtime
        .installed_schema()
        .principal_binding(BankPrincipalBinding::reference())
        .map_err(BankIdentityRuntimeBuildError::InstalledBinding)
}
