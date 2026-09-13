//! One-operation compilation authority shared by install and reinstallation.

mod capability_demand;
use capability_demand::{operation_capability_count, progression_support_fact_count};

use worth_query_declaration::facade::application_aftermath::PortableApplicationAftermathContract;
use worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaBindingIdentity,
    ApplicationSchemaMember,
};

use crate::application_aftermath::{
    install_application_aftermath, InstalledExternalEffectContract,
    WorthQueryInstalledAftermathContract,
};
use crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor;
use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;
use crate::domain_operation::{
    WorthQueryOperationGraphReadContract, WorthQueryOperationTouchContract,
};
use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
};

use super::super::contract_resolution::{
    ability_requirement_meaning_matches, operation_aftermath, operation_decision_fact_budget,
    operation_execution_posture, operation_mutation_preconditions,
    operation_projection_work_budget,
};
use super::super::installed_contract_support::{operation_authorization, operation_denial};
use super::super::{
    WorthQueryApplicationCandidateDemand, WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind,
    WorthQueryCompiledApplicationOperationContracts, WorthQueryInstalledAbilityRequirement,
    WorthQueryInstalledApplicationOperationAuthorization,
    WorthQueryInstalledApplicationOperationExecutionPosture,
    WorthQueryInstalledMutationPrecondition, WorthQueryOperationEmissionContract,
};

/// Every candidate-derived axis for one exact operation declaration.
///
/// The constructor accepts only one member set and one operation/input identity.
/// Reads, external effect, aftermath, budgets, and executable program cannot be
/// supplied independently or recombined across sibling operations.
pub(super) struct WorthQueryApplicationOperationCompilation<'a> {
    binding: ApplicationSchemaBindingIdentity,
    operation: String,
    input_type: String,
    decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    decision_fact_budget: usize,
    projection_work_budget: usize,
    execution_posture: WorthQueryInstalledApplicationOperationExecutionPosture,
    external_effect: InstalledExternalEffectContract,
    portable_aftermath: Option<PortableApplicationAftermathContract>,
    portable_contract: &'a WorthQueryPortableApplicationOperationContractRecord,
    members: &'a [ApplicationSchemaMember],
    mutation_bindings: &'a [ApplicationMutationBindingDescriptor],
}

/// Opaque, whole-operation input accepted by the compiled-contract owner.
pub(in crate::application_operation) struct WorthQuerySealedOperationContractCompilation {
    authorization: WorthQueryInstalledApplicationOperationAuthorization,
    ability_requirements: Vec<WorthQueryInstalledAbilityRequirement>,
    authored_program_width: usize,
    decision_fact_budget: usize,
    projection_work_budget: usize,
    additional_authorization_fact_count: usize,
    mutation_preconditions: Vec<WorthQueryInstalledMutationPrecondition>,
    execution_posture: WorthQueryInstalledApplicationOperationExecutionPosture,
    external_effect: InstalledExternalEffectContract,
    aftermath: Option<WorthQueryInstalledAftermathContract>,
    graph_reads: WorthQueryOperationGraphReadContract,
    touches: WorthQueryOperationTouchContract,
    emissions: WorthQueryOperationEmissionContract,
    graph_mutation_count: usize,
    candidate_demand: WorthQueryApplicationCandidateDemand,
    invariant_invocations: Vec<WorthQueryInstalledApplicationInvariantDescriptor>,
}

impl<'a> WorthQueryApplicationOperationCompilation<'a> {
    pub(super) fn resolve(
        binding: ApplicationSchemaBindingIdentity,
        members: &'a [ApplicationSchemaMember],
        mutation_bindings: &'a [ApplicationMutationBindingDescriptor],
        portable_contract: &'a WorthQueryPortableApplicationOperationContractRecord,
        operation: &str,
        input_type: &str,
    ) -> Result<Self, WorthQueryApplicationOperationInstallationDenial> {
        if !members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Operation {
                    operation: candidate,
                    input_type: candidate_input,
                } if candidate == operation && candidate_input.as_str() == input_type
            )
        }) {
            return Err(operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::OperationNotInstalled,
                operation,
            ));
        }
        let decision_fact_budget =
            operation_decision_fact_budget(members, operation).ok_or_else(|| {
                operation_denial(
                    WorthQueryApplicationOperationInstallationDenialKind::MissingDecisionFactBudget,
                    operation,
                )
            })?;
        let projection_work_budget = operation_projection_work_budget(members, operation)
            .ok_or_else(|| {
                operation_denial(
                    WorthQueryApplicationOperationInstallationDenialKind::MissingProjectionWorkBudget,
                    operation,
                )
            })?;
        if portable_contract.operation() != operation
            || portable_contract.input_type().as_str() != input_type
        {
            return Err(operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                operation,
            ));
        }
        let decision_reads = portable_contract.decision_read_targets().map_err(|()| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                operation,
            )
        })?;
        if portable_contract.authored_program_width() == 0 && decision_reads.is_empty() {
            return Err(operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::MissingProgram,
                operation,
            ));
        }
        let external_effect = install_portable_external_effect(portable_contract.external_effect());
        let portable_aftermath = operation_aftermath(members, operation)
            .map_err(|denial| operation_denial(denial.installation_kind(), operation))?;
        Ok(Self {
            binding,
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
            decision_reads,
            decision_fact_budget,
            projection_work_budget,
            execution_posture: operation_execution_posture(members, operation, input_type),
            external_effect,
            portable_aftermath,
            portable_contract,
            members,
            mutation_bindings,
        })
    }

    pub(super) fn compile_contracts(
        self,
        ability_requirements: Vec<WorthQueryInstalledAbilityRequirement>,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
    ) -> Result<
        WorthQueryCompiledApplicationOperationContracts,
        WorthQueryApplicationOperationInstallationDenial,
    > {
        if !ability_requirement_meaning_matches(
            self.members,
            &self.operation,
            &ability_requirements,
        ) {
            return Err(operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::MissingAbilityPolicy,
                &self.operation,
            ));
        }
        let authorization = operation_authorization(
            &self.operation,
            ability_requirements.len(),
            operation_capability_count(self.members, &self.operation, &self.input_type),
        )?;
        let additional_authorization_fact_count =
            progression_support_fact_count(self.members, &self.operation, &self.input_type);
        let mutation_preconditions = self.compile_mutation_preconditions(&ability_requirements)?;
        let graph_reads = super::super::contracts::install_portable_graph_reads(
            &self.binding,
            native_contracts,
            self.portable_contract.graph_reads(),
        )
        .map_err(|()| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                &self.operation,
            )
        })?;
        let (touches, graph_mutation_count) =
            super::super::contracts::install_portable_graph_touches(
            &self.binding,
            native_contracts,
            self.portable_contract.touches(),
        )
        .map_err(|()| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                &self.operation,
            )
        })?;
        let emissions = super::super::contracts::install_portable_effect_emissions(
            self.portable_contract.emissions(),
        );
        let aftermath = install_application_aftermath(&self).map_err(|_| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied,
                &self.operation,
            )
        })?;
        let sealed = WorthQuerySealedOperationContractCompilation {
            authorization,
            ability_requirements,
            authored_program_width: self.portable_contract.authored_program_width(),
            decision_fact_budget: self.decision_fact_budget,
            projection_work_budget: self.projection_work_budget,
            additional_authorization_fact_count,
            mutation_preconditions,
            execution_posture: self.execution_posture,
            external_effect: self.external_effect,
            aftermath,
            graph_reads,
            touches,
            emissions,
            graph_mutation_count,
            candidate_demand: WorthQueryApplicationCandidateDemand::derive(
                self.mutation_bindings,
                &self.operation,
                &self.input_type,
            ),
            invariant_invocations: self
                .members
                .iter()
                .filter_map(|member| {
                    let ApplicationSchemaMember::ApplicationInvariant {
                        invariant,
                        major,
                        minor,
                        execution_point,
                        maximum_work_units,
                        enforcement,
                        required_groups,
                        read_closure,
                        applicability,
                        provider,
                        cost_posture,
                    } = member
                    else {
                        return None;
                    };
                    Some(
                        WorthQueryInstalledApplicationInvariantDescriptor::from_installed_parts(
                            invariant.clone(),
                            *major,
                            *minor,
                            *execution_point,
                            *maximum_work_units,
                            *enforcement,
                            required_groups.clone(),
                            read_closure.clone(),
                            applicability.clone(),
                            provider.clone(),
                            *cost_posture,
                        ),
                    )
                })
                .collect(),
        };
        WorthQueryCompiledApplicationOperationContracts::compile(sealed).map_err(|()| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                &self.operation,
            )
        })
    }

    fn compile_mutation_preconditions(
        &self,
        abilities: &[WorthQueryInstalledAbilityRequirement],
    ) -> Result<
        Vec<WorthQueryInstalledMutationPrecondition>,
        WorthQueryApplicationOperationInstallationDenial,
    > {
        super::super::precondition_contract::compile_precondition_contract(
            operation_mutation_preconditions(self.members, &self.operation),
            &self.decision_reads,
            abilities,
        )
        .map_err(|()| {
            operation_denial(
                WorthQueryApplicationOperationInstallationDenialKind::InvalidMutationPreconditionContract,
                &self.operation,
            )
        })
    }
}

fn install_portable_external_effect(
    portable: Option<&WorthQueryPortableExternalEffectContractRecord>,
) -> InstalledExternalEffectContract {
    match portable {
        None => InstalledExternalEffectContract::None,
        Some(portable) => InstalledExternalEffectContract::Declared {
            correlation_family: portable.correlation_family().clone(),
            effect: portable.effect().to_owned(),
            rust_payload_type: portable.payload_type().clone(),
            protocol: portable.protocol().clone(),
            maximum_payload_bytes: portable.maximum_payload_bytes(),
        },
    }
}

impl super::aftermath_installation_source_seal::Sealed
    for WorthQueryApplicationOperationCompilation<'_>
{
}

impl super::WorthQueryOperationAftermathInstallationSource
    for WorthQueryApplicationOperationCompilation<'_>
{
    fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    fn operation(&self) -> &str {
        &self.operation
    }

    fn portable_decision_reads(&self) -> &[ApplicationOperationDecisionReadTarget] {
        &self.decision_reads
    }

    fn external_effect(&self) -> &InstalledExternalEffectContract {
        &self.external_effect
    }

    fn portable_aftermath(&self) -> Option<&PortableApplicationAftermathContract> {
        self.portable_aftermath.as_ref()
    }

    fn portable_reconciliation(
        &self,
    ) -> Option<&WorthQueryPortableInstalledReconciliationProcedureRecord> {
        self.portable_contract.reconciliation()
    }
}

impl WorthQuerySealedOperationContractCompilation {
    #[allow(clippy::type_complexity)]
    pub(in crate::application_operation) fn into_parts(
        self,
    ) -> (
        WorthQueryInstalledApplicationOperationAuthorization,
        Vec<WorthQueryInstalledAbilityRequirement>,
        usize,
        usize,
        usize,
        usize,
        Vec<WorthQueryInstalledMutationPrecondition>,
        WorthQueryInstalledApplicationOperationExecutionPosture,
        InstalledExternalEffectContract,
        Option<WorthQueryInstalledAftermathContract>,
        WorthQueryOperationGraphReadContract,
        WorthQueryOperationTouchContract,
        WorthQueryOperationEmissionContract,
        usize,
        WorthQueryApplicationCandidateDemand,
        Vec<WorthQueryInstalledApplicationInvariantDescriptor>,
    ) {
        (
            self.authorization,
            self.ability_requirements,
            self.authored_program_width,
            self.decision_fact_budget,
            self.projection_work_budget,
            self.additional_authorization_fact_count,
            self.mutation_preconditions,
            self.execution_posture,
            self.external_effect,
            self.aftermath,
            self.graph_reads,
            self.touches,
            self.emissions,
            self.graph_mutation_count,
            self.candidate_demand,
            self.invariant_invocations,
        )
    }
}
