use bank_domain::{
    model::BankPrincipalId,
    queries::PaymentDetailQueryBinding,
    schema::{
        ApprovePayment, ApprovePaymentOperation, ApprovedBusinessPaymentAdvance,
        ApprovedBusinessPaymentAdvanceOperation, ApprovedBusinessPaymentApproval,
        ApprovedBusinessPaymentApprovalOperation, ApprovedBusinessPaymentAuthoring,
        ApprovedBusinessPaymentAuthoringOperation, ApprovedBusinessPaymentInstanceStart,
        ApprovedBusinessPaymentInstanceStartOperation, ApprovedBusinessPaymentWorkflow,
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
        Principal,
    },
};
use worth_query_host::facade::{
    application_installation::{
        in_memory_rostered_program, in_memory_rostered_program_with_authorization_time_source,
        WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationLimits,
    },
    domain::{
        WorthQueryApplicationWorkflowResourceCeiling,
        WorthQueryApplicationWorkflowSpecInstallation, WorthQueryInstalledApplicationSchema,
        WorthQueryInstalledPrincipalBinding,
    },
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
        WorthQueryRuntimeTimeSource,
    },
    runtime::{
        WorthQueryApplicationCandidateResourceProfile, WorthQueryApplicationQueryResourceProfile,
    },
};

use super::{BankGraphSeed, BankIdentityRuntime};
use crate::{
    application_definition::{validated_bank_application, validated_bank_application_p1},
    error::BankIdentityRuntimeBuildError,
    graph_bootstrap::bind_bank_world_with_estate,
    principal_seed::PreparedBankPrincipalSeed,
};

type InstalledBankPrincipalBinding = WorthQueryInstalledPrincipalBinding<
    BankSchema,
    BankPrincipalBinding,
    ExternalPrincipalMapping,
    Principal,
    BankPrincipalId,
    BankPrincipalIdBinding,
>;

pub(super) enum BankAuthorizationTimeInstallation {
    System,
    Installed(Box<dyn WorthQueryRuntimeTimeSource>),
}

pub(super) fn install_prepared(
    seeds: Vec<PreparedBankPrincipalSeed>,
    world: Option<BankGraphSeed>,
    authorization_time: BankAuthorizationTimeInstallation,
) -> Result<BankIdentityRuntime, BankIdentityRuntimeBuildError> {
    let program = validated_bank_application()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let successor = validated_bank_application_p1()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let declaration =
        BankSchema::declaration().map_err(BankIdentityRuntimeBuildError::SchemaDeclaration)?;
    let limits = bank_application_limits();
    let mut invariant_projection = None;
    let initialize =
        |graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
         installed: &WorthQueryInstalledApplicationSchema<BankSchema>| {
            bind_principals(graph, installed, seeds)?;
            bind_world_truth(graph, world.as_ref())?;
            invariant_projection = Some(graph.retain_invariant_projection_authority());
            Ok(())
        };
    let runtime = match authorization_time {
        BankAuthorizationTimeInstallation::System => in_memory_rostered_program(
            program,
            WorthQueryApplicationProgramRoster::new().support(successor),
            declaration,
            ((), (), ()),
            limits,
            initialize,
        ),
        BankAuthorizationTimeInstallation::Installed(source) => {
            in_memory_rostered_program_with_authorization_time_source(
                program,
                WorthQueryApplicationProgramRoster::new().support(successor),
                declaration,
                ((), (), ()),
                limits,
                initialize,
                source,
            )
        }
    }
    .map_err(BankIdentityRuntimeBuildError::ApplicationInstallation)?;
    let invariant_projection = invariant_projection
        .expect("program construction invokes the initializer before publication");
    let binding = resolve_installed_principal_binding(runtime.runtime())?;
    let workflow = WorthQueryApplicationWorkflowSpecInstallation::<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        crate::application_definition::BankApplication,
    >::begin(
        runtime.runtime().installed_schema(),
        runtime.installed_program(),
        payment_workflow_resources(),
    )
    .operation::<ApprovedBusinessPaymentAuthoringOperation, ApprovePayment>()
    .map_err(workflow_error)?
    .operation::<ApprovePaymentOperation, ApprovePayment>()
    .map_err(workflow_error)?
    .assessment::<PaymentDetailQueryBinding>()
    .map_err(workflow_error)?
    .approval::<
        ApprovedBusinessPaymentApproval,
        ApprovedBusinessPaymentApprovalOperation,
        ApprovePayment,
    >()
    .map_err(workflow_error)?
    .authoring_capability::<
        ApprovedBusinessPaymentAuthoring,
        ApprovedBusinessPaymentAuthoringOperation,
        ApprovePayment,
    >()
    .map_err(workflow_error)?
    .instance_start_capability::<
        ApprovedBusinessPaymentInstanceStart,
        ApprovedBusinessPaymentInstanceStartOperation,
        ApprovePayment,
    >()
    .map_err(workflow_error)?
    .advance_capability::<
        ApprovedBusinessPaymentAdvance,
        ApprovedBusinessPaymentAdvanceOperation,
        ApprovePayment,
    >()
    .map_err(workflow_error)?
    .finish()
    .map_err(workflow_error)?;
    let runtime = runtime
        .retain_workflow_spec(workflow)
        .map_err(workflow_error)?;
    Ok(BankIdentityRuntime {
        runtime,
        binding,
        invariant_projection,
    })
}

fn payment_workflow_resources() -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        4,
        64 * 1_024,
        32,
        128,
        256 * 1_024,
    )
    .expect("approved-payment workflow resources are nonzero")
}

fn workflow_error(error: impl std::fmt::Debug) -> BankIdentityRuntimeBuildError {
    BankIdentityRuntimeBuildError::WorkflowInstallation(format!("{error:?}"))
}

pub(crate) fn bank_application_limits() -> WorthQueryInMemoryApplicationLimits {
    let queries = WorthQueryApplicationQueryResourceProfile::bounded(32_768, 262_144, 32_768, 64)
        .expect("bank application-query resource profile is statically non-zero");
    WorthQueryInMemoryApplicationLimits::new(
        super::product_world_resources::bank_product_world_resources(),
        WorthQueryApplicationCandidateResourceProfile::bounded(4_096, 2 * 1_024 * 1_024, 1_048_576)
            .expect("bank application candidate limits are statically non-zero"),
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
    )
}

fn resolve_installed_principal_binding(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
) -> Result<InstalledBankPrincipalBinding, BankIdentityRuntimeBuildError> {
    runtime
        .installed_schema()
        .principal_binding(BankPrincipalBinding::reference())
        .map_err(BankIdentityRuntimeBuildError::InstalledBinding)
}
