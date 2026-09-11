use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect, ApplicationSchema,
    ApplicationSchemaMember, WorthQueryInstalledAbilityRequirement,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledAuthorizationPath,
};
use worth_relational::facade::authorization::RelationalAuthorizationPathPlan;
use worth_runtime_bridge::facade::{
    BridgeAuthorizationClauseContract, BridgeAuthorizationCorrespondenceIdentity,
    BridgeAuthorizationInstallationBatch, BridgeAuthorizationInstallationRequest,
    BridgeAuthorizationRequirementContract, BridgeAuthorizationRuleContract,
    BridgeAuthorizationRuleEffect, BridgeAuthorizationRuntime,
};

use super::capability_registry::{
    WorthQueryCapabilityPlanCompilationEvidence, WorthQueryElevationLifecycleOperationRole,
    WorthQueryInstalledCapabilityPlan, WorthQueryInstalledCapabilityRegistry,
};
use super::lowering::lower_authorization_path;
use super::{authorization_denial, WorthQueryOperationAuthorizationDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PolicyKey {
    ability: String,
    scope_entity: String,
    policy: String,
}

pub(super) struct WorthQueryInstalledAuthorizationPolicy {
    correspondence: BridgeAuthorizationCorrespondenceIdentity,
    bridge_rule_bindings: Vec<BridgeRuleBinding>,
    bridge_path_bindings: Vec<BridgePathBinding>,
    relational_paths: Vec<RelationalAuthorizationPathPlan>,
    principal_kind: worth_relational::facade::identity::KindId,
    scope_kind: worth_relational::facade::identity::KindId,
}

impl WorthQueryInstalledAuthorizationPolicy {
    pub(super) const fn correspondence(&self) -> BridgeAuthorizationCorrespondenceIdentity {
        self.correspondence
    }

    pub(super) fn bridge_rule_bindings(&self) -> &[BridgeRuleBinding] {
        &self.bridge_rule_bindings
    }

    pub(super) fn bridge_path_bindings(&self) -> &[BridgePathBinding] {
        &self.bridge_path_bindings
    }

    pub(super) fn relational_paths(&self) -> &[RelationalAuthorizationPathPlan] {
        &self.relational_paths
    }

    pub(super) const fn principal_kind(&self) -> worth_relational::facade::identity::KindId {
        self.principal_kind
    }

    pub(super) const fn scope_kind(&self) -> worth_relational::facade::identity::KindId {
        self.scope_kind
    }
}

pub(super) struct BridgeRuleBinding {
    rule: BridgeAuthorizationRuleContract,
    path_indices: Vec<usize>,
}

impl BridgeRuleBinding {
    pub(super) const fn rule(&self) -> &BridgeAuthorizationRuleContract {
        &self.rule
    }

    pub(super) fn path_indices(&self) -> &[usize] {
        &self.path_indices
    }
}

#[derive(Clone, Copy)]
pub(super) struct BridgePathBinding {
    pub(super) identity: [u8; 32],
    rule_effect: BridgeAuthorizationRuleEffect,
}

pub(in crate::domain_computation) struct WorthQueryInstalledAuthorizationRegistry {
    bridge: BridgeAuthorizationRuntime,
    policies: BTreeMap<PolicyKey, WorthQueryInstalledAuthorizationPolicy>,
    capabilities: WorthQueryInstalledCapabilityRegistry,
}

impl WorthQueryInstalledAuthorizationRegistry {
    pub(in crate::domain_computation) fn compile<Schema>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
    {
        let mut bridge_installation = BridgeAuthorizationInstallationBatch::new();
        let mut policies = BTreeMap::new();
        for member in schema.installed_declaration().members() {
            let ApplicationSchemaMember::AbilityPolicy {
                ability,
                scope_entity,
                policy,
                paths,
            } = member
            else {
                continue;
            };
            let key = PolicyKey {
                ability: ability.clone(),
                scope_entity: scope_entity.clone(),
                policy: policy.clone(),
            };
            let installed = compile_policy(schema, layout, &mut bridge_installation, &key, paths)?;
            if policies.insert(key, installed).is_some() {
                return Err(authorization_denial(
                    policy,
                    "duplicate installed authorization policy",
                ));
            }
        }
        let capabilities = WorthQueryInstalledCapabilityRegistry::compile(
            schema,
            layout,
            &mut bridge_installation,
        )?;
        let mut bridge = BridgeAuthorizationRuntime::new();
        bridge
            .install_batch(bridge_installation)
            .map_err(|denial| {
                authorization_denial(denial.subject(), "Bridge rejected installation batch")
            })?;
        Ok(Self {
            bridge,
            policies,
            capabilities,
        })
    }

    pub(super) fn policy(
        &self,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<&WorthQueryInstalledAuthorizationPolicy, WorthQueryOperationAuthorizationDenial>
    {
        let key = PolicyKey {
            ability: requirement.ability().to_string(),
            scope_entity: requirement.scope_entity().to_string(),
            policy: requirement.policy().to_string(),
        };
        self.policies
            .get(&key)
            .filter(|installed| {
                installed.bridge_path_bindings.len() == requirement.policy_paths().len()
                    && installed
                        .bridge_path_bindings
                        .iter()
                        .zip(requirement.policy_paths())
                        .all(|(binding, installed_path)| {
                            &binding.identity == installed_path.identity().bytes()
                                && binding.rule_effect
                                    == lower_rule_effect(installed_path.path().effect())
                        })
            })
            .ok_or_else(|| {
                authorization_denial(
                    requirement.policy(),
                    "operation requirement does not match installed policy",
                )
            })
    }

    pub(in crate::domain_computation) const fn bridge(&self) -> &BridgeAuthorizationRuntime {
        &self.bridge
    }

    pub(super) fn capability_plan<Schema, Capability, Operation, Input>(
        &self,
        capability: &worth_query_installation::facade::WorthQueryInstalledApplicationCapability<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Option<&WorthQueryInstalledCapabilityPlan> {
        self.capabilities.plan(capability)
    }

    pub(super) fn capability_plan_by_identity(
        &self,
        capability_identity: &[u8; 32],
    ) -> Option<&WorthQueryInstalledCapabilityPlan> {
        self.capabilities.plan_by_identity(capability_identity)
    }

    pub(in crate::domain_computation) fn elevation_lifecycle_operation<Schema, Operation>(
        &self,
        operation: &str,
        input_type: &str,
    ) -> Result<
        Option<(
            [u8; 32],
            [u8; 32],
            WorthQueryElevationLifecycleOperationRole,
        )>,
        (),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        self.capabilities
            .elevation_lifecycle_operation::<Schema, Operation>(operation, input_type)
    }

    pub(in crate::domain_computation) const fn capability_compilation(
        &self,
    ) -> WorthQueryCapabilityPlanCompilationEvidence {
        self.capabilities.compilation()
    }
}

fn compile_policy<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &WorthQueryPrimaryGraphLayout,
    bridge_installation: &mut BridgeAuthorizationInstallationBatch,
    key: &PolicyKey,
    paths: &[ApplicationAuthorizationPath],
) -> Result<WorthQueryInstalledAuthorizationPolicy, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let principal_entity = paths
        .first()
        .map(ApplicationAuthorizationPath::principal_entity)
        .ok_or_else(|| {
            authorization_denial(&key.policy, "installed authorization policy is empty")
        })?;
    let principal_kind = layout.entity_kind(principal_entity).ok_or_else(|| {
        authorization_denial(principal_entity, "policy principal kind is not installed")
    })?;
    let scope_kind = layout.entity_kind(&key.scope_entity).ok_or_else(|| {
        authorization_denial(&key.scope_entity, "policy scope kind is not installed")
    })?;
    let installed_paths = schema
        .installed_ability_requirement(&key.ability, &key.scope_entity)
        .ok_or_else(|| {
            authorization_denial(
                &key.policy,
                "installed authorization path identity is unavailable",
            )
        })?;
    let compiled_bridge = compile_bridge_policy(installed_paths.policy_paths());
    let relational_paths = paths
        .iter()
        .map(|path| lower_authorization_path(layout, path))
        .collect::<Result<Vec<_>, _>>()?;
    let correspondence = bridge_installation
        .add(BridgeAuthorizationInstallationRequest::new(
            installed_paths.identity(),
            super::bridge_authorization_binding_identity(&schema.binding_identity()),
            &key.ability,
            &key.scope_entity,
            &key.policy,
            compiled_bridge
                .rule_bindings
                .iter()
                .map(|binding| binding.rule.clone()),
        ))
        .map_err(|denial| authorization_denial(denial.subject(), "Bridge rejected policy"))?;
    Ok(WorthQueryInstalledAuthorizationPolicy {
        correspondence,
        bridge_rule_bindings: compiled_bridge.rule_bindings,
        bridge_path_bindings: compiled_bridge.path_bindings,
        relational_paths,
        principal_kind,
        scope_kind,
    })
}

struct CompiledBridgePolicy {
    rule_bindings: Vec<BridgeRuleBinding>,
    path_bindings: Vec<BridgePathBinding>,
}

fn compile_bridge_policy(paths: &[WorthQueryInstalledAuthorizationPath]) -> CompiledBridgePolicy {
    let bindings = paths
        .iter()
        .map(|path| BridgePathBinding {
            identity: *path.identity().bytes(),
            rule_effect: lower_rule_effect(path.path().effect()),
        })
        .collect::<Vec<_>>();
    let required = path_indices_for_effect(&bindings, BridgeAuthorizationRuleEffect::Required);
    let prohibited = path_indices_for_effect(&bindings, BridgeAuthorizationRuleEffect::Prohibited);
    let mut rule_bindings = vec![bind_bridge_rule(
        BridgeAuthorizationRuleEffect::Required,
        required,
        &bindings,
    )];
    if !prohibited.is_empty() {
        rule_bindings.push(bind_bridge_rule(
            BridgeAuthorizationRuleEffect::Prohibited,
            prohibited,
            &bindings,
        ));
    }
    CompiledBridgePolicy {
        rule_bindings,
        path_bindings: bindings,
    }
}

fn bind_bridge_rule(
    effect: BridgeAuthorizationRuleEffect,
    path_indices: Vec<usize>,
    bindings: &[BridgePathBinding],
) -> BridgeRuleBinding {
    BridgeRuleBinding {
        rule: bridge_rule(effect, &path_indices, bindings),
        path_indices,
    }
}

fn path_indices_for_effect(
    bindings: &[BridgePathBinding],
    effect: BridgeAuthorizationRuleEffect,
) -> Vec<usize> {
    bindings
        .iter()
        .enumerate()
        .filter_map(|(index, binding)| (binding.rule_effect == effect).then_some(index))
        .collect()
}

fn bridge_rule(
    effect: BridgeAuthorizationRuleEffect,
    indices: &[usize],
    bindings: &[BridgePathBinding],
) -> BridgeAuthorizationRuleContract {
    BridgeAuthorizationRuleContract::all(
        effect,
        [BridgeAuthorizationRequirementContract::any(
            indices
                .iter()
                .map(|index| BridgeAuthorizationClauseContract::new(bindings[*index].identity)),
        )],
    )
}

const fn lower_rule_effect(
    effect: ApplicationAuthorizationPathEffect,
) -> BridgeAuthorizationRuleEffect {
    match effect {
        ApplicationAuthorizationPathEffect::Allow => BridgeAuthorizationRuleEffect::Required,
        ApplicationAuthorizationPathEffect::Deny => BridgeAuthorizationRuleEffect::Prohibited,
    }
}
