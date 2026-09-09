use std::any::TypeId;
use std::marker::PhantomData;

use crate::basis_lifecycle::BasisOperationLane;

mod commit_posture;
mod conditional_inventory;
mod graph_contract;

use super::authority_shape::WorthQueryBoundAuthorityShapeProofs;
use super::execution_support::{
    lower_installed_execution_support, WorthQueryInstalledRuntimeProviders,
};
use commit_posture::admit_commit_posture;
use conditional_inventory::{
    admit_conditional_inventory, ConditionalInventoryAdmission, ConditionalInventoryOwner,
};
use graph_contract::admit_graph_contract;

use super::{
    WorthQueryBoundAuthoritySet, WorthQueryBoundCommitPosture, WorthQueryBoundDomainOperation,
    WorthQueryBoundGraphParticipation, WorthQueryBoundRequiredDomain,
    WorthQueryInstalledOperatingWorld, WorthQueryOperationBindingCounters,
    WorthQueryOperationBindingDenial, WorthQueryOperationBindingDenialKind,
};
use crate::domain_installation::{
    WorthQueryInstalledDomainHandle, WorthQueryOperationEffectContract,
    WorthQueryOperationTouchContract,
};

pub struct WorthQueryOperationFamilyView<'view, 'runtime, F, L: BasisOperationLane> {
    world: &'view WorthQueryInstalledOperatingWorld<'runtime, L>,
    _family: PhantomData<fn() -> F>,
}

impl<'view, 'runtime, F, L: BasisOperationLane>
    WorthQueryOperationFamilyView<'view, 'runtime, F, L>
{
    pub(super) fn new(world: &'view WorthQueryInstalledOperatingWorld<'runtime, L>) -> Self {
        Self {
            world,
            _family: PhantomData,
        }
    }

    pub fn bind<D: 'static, O: 'static>(
        &self,
        domain: &WorthQueryInstalledDomainHandle<D>,
        _operation_marker: O,
    ) -> Result<WorthQueryBoundDomainOperation<D, O, F, L>, WorthQueryOperationBindingDenial>
    where
        F: 'static,
    {
        let operation = resolve_operation(self.world, domain)?;
        let mut counters = WorthQueryOperationBindingCounters {
            authority_checks: operation.lookup_counters().authority_checks,
            operation_lookups: operation.lookup_counters().indexed_operation_lookups,
            graph_binding_lookups: operation.lookup_counters().graph_binding_lookups,
            ..WorthQueryOperationBindingCounters::default()
        };
        let conditional_nodes = admit_bound_conditionals(self.world, &operation, &mut counters)?;
        admit_basis_execution::<D, O, F, L>(self.world, &operation, counters)?;
        let mut graphs = bind_graph_authorities(self.world, &operation, &mut counters)?;
        let mut required_domains = bind_required_domains::<D, O, F, L>(self.world, &mut counters)?;
        counters.authority_shape_admissions += 1;
        let shape_proofs =
            WorthQueryBoundAuthorityShapeProofs::admit(&mut graphs, &mut required_domains)
                .map_err(|_| {
                    WorthQueryOperationBindingDenial::new(
                        WorthQueryOperationBindingDenialKind::IncoherentAuthoritySet,
                        "bound graph and required-domain roles must be canonical and unique",
                        counters,
                    )
                })?;
        counters.commit_posture_classifications += 1;
        let commit_posture = admit_commit_posture(&operation, &graphs, &mut counters)?;
        counters.planning_steps += 1;
        counters.executor_route_lookups += 1;
        let executor = self.world.runtime.domain_operation_executor::<D, O, F>();
        counters.workflow_executor_route_lookups += 1;
        let workflow_executor = self.world.runtime.workflow_stage_executor::<D, O, F>();
        counters.parallel_admission_route_lookups += 1;
        let workflow_parallel_admission_provider = self
            .world
            .runtime
            .workflow_parallel_admission_provider::<D, O, F>();
        let execution_closure = lower_installed_execution_support(
            operation.definition(),
            &graphs,
            commit_posture,
            &conditional_nodes,
            WorthQueryInstalledRuntimeProviders {
                direct: executor.as_ref(),
                workflow_graph: operation.workflow_graph().cloned(),
                workflow: workflow_executor.as_ref(),
                parallel: workflow_parallel_admission_provider.as_ref(),
            },
        )
        .map_err(|detail| {
            WorthQueryOperationBindingDenial::new(
                WorthQueryOperationBindingDenialKind::ExecutionAuthority,
                detail,
                counters,
            )
        })?;
        counters.execution_authority_admissions += 1;
        let execution_authority = bind_execution_authority(
            self.world,
            &operation,
            &graphs,
            &required_domains,
            commit_posture,
            execution_closure.support,
            counters,
        )?;
        Ok(WorthQueryBoundDomainOperation::mint(
            operation,
            self.world.basis.clone(),
            execution_authority,
            WorthQueryBoundAuthoritySet {
                graph_participations: graphs,
                required_domains,
                commit_posture,
                shape_proofs,
            },
            self.world.runtime.consumer_support_profile().clone(),
            execution_closure.providers,
            conditional_nodes,
            counters,
        ))
    }
}

fn resolve_operation<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    domain: &WorthQueryInstalledDomainHandle<D>,
) -> Result<
    crate::domain_installation::WorthQueryInstalledDomainOperation<D, O, F>,
    WorthQueryOperationBindingDenial,
> {
    world
        .runtime
        .resolve_installed_operation::<D, O, F>(domain)
        .map_err(|denial| {
            let kind = match denial.kind() {
                crate::domain_installation::WorthQueryInstalledDomainOperationLookupDenialKind::DomainAuthority => WorthQueryOperationBindingDenialKind::DomainAuthority,
                crate::domain_installation::WorthQueryInstalledDomainOperationLookupDenialKind::OperationNotInstalled => WorthQueryOperationBindingDenialKind::OperationNotInstalled,
            };
            WorthQueryOperationBindingDenial::new(
                kind,
                "installed operation lookup failed",
                WorthQueryOperationBindingCounters {
                    authority_checks: denial.counters().authority_checks,
                    operation_lookups: denial.counters().indexed_operation_lookups,
                    ..WorthQueryOperationBindingCounters::default()
                },
            )
        })
}

fn admit_bound_conditionals<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    operation: &crate::domain_installation::WorthQueryInstalledDomainOperation<D, O, F>,
    counters: &mut WorthQueryOperationBindingCounters,
) -> Result<
    Vec<std::sync::Arc<crate::domain_installation::WorthQueryInstalledConditionalNode>>,
    WorthQueryOperationBindingDenial,
> {
    counters.conditional_lowering_lookups += 1;
    let conditional_nodes = world.runtime.conditional_nodes::<D, O, F>();
    if !conditional_nodes.is_empty() && world.basis.product().is_err() {
        return Err(WorthQueryOperationBindingDenial::new(
            WorthQueryOperationBindingDenialKind::ProductBasisRequired,
            "conditional operations require a retained product selection",
            *counters,
        ));
    }
    counters.conditional_lowerings_retained = conditional_nodes.len();
    let admission = admit_conditional_inventory(
        operation.definition(),
        &conditional_nodes,
        ConditionalInventoryOwner {
            runtime_authority: operation.domain_authority().runtime_authority().as_u64(),
            installation_generation: operation.installation_generation().ordinal(),
        },
        counters,
    );
    let kind = match admission {
        ConditionalInventoryAdmission::Admitted => {
            return Ok(conditional_nodes);
        }
        ConditionalInventoryAdmission::Missing => {
            WorthQueryOperationBindingDenialKind::ConditionalLoweringNotInstalled
        }
        ConditionalInventoryAdmission::Drifted => {
            WorthQueryOperationBindingDenialKind::ConditionalLoweringDrift
        }
    };
    Err(WorthQueryOperationBindingDenial::new(
        kind,
        "installed conditional lowerings differ from the portable declaration or owner",
        *counters,
    ))
}

fn admit_basis_execution<D, O, F, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    operation: &crate::domain_installation::WorthQueryInstalledDomainOperation<D, O, F>,
    counters: WorthQueryOperationBindingCounters,
) -> Result<(), WorthQueryOperationBindingDenial> {
    let semantics = operation.definition().semantics();
    let requires_primary_read = semantics.graph_reads.domain_roles().iter().any(|read| {
        read.participation
            == crate::domain_installation::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph
    });
    if requires_primary_read
        && world.basis.lane().normalized().family()
            != crate::basis_lifecycle::BasisFamily::CurrentHead
    {
        return Err(WorthQueryOperationBindingDenial::new(
            WorthQueryOperationBindingDenialKind::BasisExecutionUnsupported,
            "installed primary reads currently lower only through an exact current-head basis",
            counters,
        ));
    }
    let mutation_required = matches!(
        semantics.effects,
        WorthQueryOperationEffectContract::Declared { .. }
    ) || matches!(
        semantics.touches,
        WorthQueryOperationTouchContract::Declared { .. }
    );
    if mutation_required
        && L::lane_name()
            != <crate::basis_lifecycle::MutationPreparationLaneWitness as BasisOperationLane>::lane_name()
    {
        return Err(WorthQueryOperationBindingDenial::new(
            WorthQueryOperationBindingDenialKind::BasisLaneInsufficient,
            "touch/effect operations require an admitted mutation-preparation basis lane",
            counters,
        ));
    }
    Ok(())
}

fn bind_graph_authorities<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    operation: &crate::domain_installation::WorthQueryInstalledDomainOperation<D, O, F>,
    counters: &mut WorthQueryOperationBindingCounters,
) -> Result<Vec<WorthQueryBoundGraphParticipation>, WorthQueryOperationBindingDenial> {
    let bindings = world
        .runtime
        .installed_domain_execution_index()
        .domain_operation_graph_bindings(TypeId::of::<D>(), TypeId::of::<O>(), TypeId::of::<F>());
    let mut graphs = Vec::with_capacity(bindings.len());
    for binding in bindings {
        counters.graph_participation_lookups += 1;
        let record = world
            .runtime
            .installed_graph_participation(binding.graph_marker)
            .map_err(|_| {
                WorthQueryOperationBindingDenial::new(
                    WorthQueryOperationBindingDenialKind::GraphParticipationNotInstalled,
                    &binding.role,
                    *counters,
                )
            })?;
        if record.definition.role != binding.role {
            return Err(WorthQueryOperationBindingDenial::new(
                WorthQueryOperationBindingDenialKind::GraphRoleMismatch,
                &binding.role,
                *counters,
            ));
        }
        admit_graph_contract(operation, &binding.role, &record, counters)?;
        graphs.push(WorthQueryBoundGraphParticipation {
            role: binding.role.clone(),
            record,
        });
    }
    Ok(graphs)
}

fn bind_required_domains<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    counters: &mut WorthQueryOperationBindingCounters,
) -> Result<Vec<WorthQueryBoundRequiredDomain>, WorthQueryOperationBindingDenial> {
    let bindings = world
        .runtime
        .installed_domain_execution_index()
        .domain_operation_required_domains(TypeId::of::<D>(), TypeId::of::<O>(), TypeId::of::<F>());
    let mut domains = Vec::with_capacity(bindings.len());
    for binding in bindings {
        counters.required_domain_lookups += 1;
        let authority = world
            .runtime
            .installed_domain_authority_by_marker(binding.domain_marker)
            .ok_or_else(|| {
                WorthQueryOperationBindingDenial::new(
                    WorthQueryOperationBindingDenialKind::RequiredDomainNotInstalled,
                    &binding.role,
                    *counters,
                )
            })?;
        domains.push(WorthQueryBoundRequiredDomain {
            role: binding.role.clone(),
            authority,
        });
    }
    Ok(domains)
}

fn bind_execution_authority<D, O, F, L: BasisOperationLane>(
    world: &WorthQueryInstalledOperatingWorld<'_, L>,
    operation: &crate::domain_installation::WorthQueryInstalledDomainOperation<D, O, F>,
    graphs: &[WorthQueryBoundGraphParticipation],
    required_domains: &[WorthQueryBoundRequiredDomain],
    commit_posture: WorthQueryBoundCommitPosture,
    installed_support:
        worth_query_execution::facade::domain_computation::WorthQueryInstalledOperationExecutionSupport,
    counters: WorthQueryOperationBindingCounters,
) -> Result<
    worth_query_execution::facade::runtime::WorthQueryExecutionBoundOperationAuthority,
    WorthQueryOperationBindingDenial,
> {
    world
        .runtime
        .query_execution_runtime()
        .bind_domain_operation(
            world.runtime.query_execution_installation_authority(),
            operation.operation_authority(),
            world.basis.lane(),
            &graphs
                .iter()
                .map(|binding| binding.record.installation_authority.as_ref())
                .collect::<Vec<_>>(),
            &required_domains
                .iter()
                .map(|binding| {
                    (
                        binding.role.as_str(),
                        binding.authority.portable_authority(),
                    )
                })
                .collect::<Vec<_>>(),
            execution_commit_posture(commit_posture),
            installed_support,
        )
        .map_err(|denial| {
            WorthQueryOperationBindingDenial::new(
                WorthQueryOperationBindingDenialKind::ExecutionAuthority,
                format!("execution authority rejected bound operation: {denial:?}"),
                counters,
            )
        })
}

fn execution_commit_posture(
    posture: WorthQueryBoundCommitPosture,
) -> worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture {
    use worth_query_execution::facade::domain_computation::WorthQueryExecutionCommitPosture;
    match posture {
        WorthQueryBoundCommitPosture::ReadOnly => WorthQueryExecutionCommitPosture::ReadOnly,
        WorthQueryBoundCommitPosture::Atomic => WorthQueryExecutionCommitPosture::Atomic,
        WorthQueryBoundCommitPosture::Compensated => WorthQueryExecutionCommitPosture::Compensated,
    }
}
