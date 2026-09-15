use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    install_application_program, WorthQueryInstalledApplicationProgram,
};

use super::{
    in_memory_with_program, WorthQueryInMemoryApplicationDenial,
    WorthQueryInMemoryApplicationLimits,
};

mod bindings;
mod demand;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
pub use bindings::WorthQueryProgramConnectionBindings;
pub use demand::{
    WorthQueryAdmittedProgramOutput, WorthQueryProgramOutputAdvance, WorthQuerySettledProgramOutput,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProgramConnectionBindingIdentity {
    node: std::any::TypeId,
    binding: std::any::TypeId,
    source_feature: std::any::TypeId,
    source_port: std::any::TypeId,
    target_feature: std::any::TypeId,
    target_port: std::any::TypeId,
}

fn require_complete_execution_bindings(
    expected: &[ProgramConnectionBindingIdentity],
    actual: &[ProgramConnectionBindingIdentity],
    subject: &str,
) -> Result<(), worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenial> {
    if expected.len() != actual.len() || !expected.iter().all(|binding| actual.contains(binding)) {
        return Err(
            worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenial::new(
                worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenialKind::IncompleteExecutionBindings,
                subject,
            ),
        );
    }
    Ok(())
}

/// Runtime paired with the exact validated program that governed installation.
pub struct WorthQueryProgramApplicationRuntime<Schema, Program> {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    program: WorthQueryInstalledApplicationProgram<Schema, Program>,
}

/// Internal cross-crate port used by Query publication orchestration.
/// The host facade does not export this constructor or type.
pub struct WorthQueryProgramExecutionPort<'application, Schema, Program> {
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
}

pub fn program_execution_port<Schema, Program>(
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> WorthQueryProgramExecutionPort<'_, Schema, Program> {
    WorthQueryProgramExecutionPort { application }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    pub const fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        &self.runtime
    }

    pub const fn installed_program(
        &self,
    ) -> &WorthQueryInstalledApplicationProgram<Schema, Program> {
        &self.program
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub fn program_recovery_count_for_test(&self) -> usize {
        self.runtime.output_demands.program_recovery_count()
    }
}

impl<Schema, Program> std::ops::Deref for WorthQueryProgramApplicationRuntime<Schema, Program> {
    type Target = WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

/// Validates and installs program meaning before exposing its application runtime.
pub fn in_memory_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition<Contributions = Program::Contributions>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryProgramConnectionBindings<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let mut runtime =
        in_memory_with_program(declaration, configuration, limits, initial_state, true)?;
    let installed = install_application_program(program, runtime.installed_schema())
        .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    let bindings =
        Program::Connections::bind().map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    let expected = installed
        .connections()
        .iter()
        .filter(|connection| {
            connection.role()
                != worth_query_declaration::facade::application_program::ApplicationProgramConnectionRole::Unavailable
        })
        .map(|connection| ProgramConnectionBindingIdentity {
            node: connection.node_type(),
            binding: connection.binding_type(),
            source_feature: connection.source_feature_type(),
            source_port: connection.source_port_type(),
            target_feature: connection.target_feature_type(),
            target_port: connection.target_port_type(),
        })
        .collect::<Vec<_>>();
    let actual = bindings
        .connections()
        .iter()
        .map(|connection| ProgramConnectionBindingIdentity {
            node: connection.node_type,
            binding: connection.binding_type,
            source_feature: connection.source_feature_type,
            source_port: connection.source_port_type,
            target_feature: connection.target_feature_type,
            target_port: connection.target_port_type,
        })
        .collect::<Vec<_>>();
    require_complete_execution_bindings(&expected, &actual, Program::IDENTITY.as_str())
        .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    bindings::require_matching_root_demands(&bindings)
        .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    runtime
        .program_required_bindings
        .extend(bindings.required_sources());
    Ok(WorthQueryProgramApplicationRuntime {
        runtime,
        program: installed,
    })
}

impl<Schema, Program> WorthQueryProgramExecutionPort<'_, Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn retain_program_recovery<Inventory, Root, Demand>(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        demand: &Demand,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>
    where
        Inventory: 'static,
        Root: 'static,
        Demand: Clone + Send + Sync + 'static,
    {
        self.application
            .runtime
            .output_demands
            .retain_program_recovery::<Inventory, Root, Demand>(
                receipt,
                demand,
                &prepared.preparation,
            )
    }

    pub fn recover_program_demand<Inventory, Root, Demand>(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<Demand, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>
    where
        Inventory: 'static,
        Root: 'static,
        Demand: Clone + 'static,
    {
        if receipt.provider_runtime_instance_id()
            != self
                .application
                .runtime
                .relational_branch_identity
                .runtime_instance_id()
        {
            return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "program recovery receipt belongs to another provider runtime",
            ));
        }
        match self
            .application
            .runtime
            .product_runtime
            .admit_product_occurrence(receipt.product_branch().occurrence())
        {
            Ok(lease) => drop(lease),
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::RetiredBranch)
            | Err(crate::basis::WorthQueryProductBranchAdmissionDenial::IncarnationChanged) => {
                return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                    "program recovery source product occurrence is retired",
                ));
            }
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ForeignOwner) => {
                return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    "program recovery source product belongs to another runtime",
                ));
            }
            Err(_) => {
                return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "program recovery source product is unavailable",
                ));
            }
        }
        self.application
            .runtime
            .output_demands
            .recover_program_demand::<Inventory, Root, Demand>(receipt)
    }

    pub fn release_program_recovery(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        self.application
            .runtime
            .output_demands
            .release_program_recovery(source_commit);
    }

    pub fn compare_and_commit_required_output_source<Source>(
        &self,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Source::Operation,
            Source::Input,
            <Source::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
            Option<(
                crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
                std::sync::Arc<
                    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
                >,
            )>,
        ),
        crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure,
    >
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Source::Input: Clone + Send + Sync + 'static,
    {
        let source_preparation = self
            .application
            .runtime
            .output_demands
            .begin_source_preparation(program.product_branch().occurrence());
        match self
            .application
            .runtime
            .compare_and_commit_application_for_required_output_source(program, idempotency)
        {
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                mut receipt,
            ) => {
                let descriptive = receipt.clone();
                let Some(change) = receipt.take_performed_relational_product_change() else {
                    return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                        receipt: descriptive,
                        denial: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                            "committed required-output source has no performed-change carrier",
                        ),
                    });
                };
                let prepared = match self
                    .application
                    .runtime
                    .retain_required_output_source(receipt, change, source_preparation)
                {
                    Ok(prepared) => prepared,
                    Err(denial) => {
                        return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                            receipt: descriptive,
                            denial,
                        })
                    }
                };
                Ok((
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                        descriptive,
                    ),
                    Some(prepared),
                ))
            }
            outcome => Ok((outcome, None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity<T: 'static>() -> ProgramConnectionBindingIdentity {
        ProgramConnectionBindingIdentity {
            node: std::any::TypeId::of::<T>(),
            binding: std::any::TypeId::of::<()>(),
            source_feature: std::any::TypeId::of::<()>(),
            source_port: std::any::TypeId::of::<()>(),
            target_feature: std::any::TypeId::of::<()>(),
            target_port: std::any::TypeId::of::<()>(),
        }
    }

    #[test]
    fn every_available_connection_requires_one_exact_execution_binding() {
        for (expected, actual) in [
            (
                vec![identity::<u8>(), identity::<u16>()],
                vec![identity::<u8>()],
            ),
            (
                vec![identity::<u8>()],
                vec![identity::<u8>(), identity::<u16>()],
            ),
            (vec![identity::<u8>()], vec![identity::<u16>()]),
        ] {
            let denial = require_complete_execution_bindings(&expected, &actual, "program")
                .expect_err("execution bindings must match the available declaration exactly");
            assert_eq!(
                denial.kind(),
                worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenialKind::IncompleteExecutionBindings
            );
            assert_eq!(denial.subject(), "program");
        }
    }
}
