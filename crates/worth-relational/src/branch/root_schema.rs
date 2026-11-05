use std::sync::Arc;

use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, PortableAspectContractLookup,
};

use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
    SchemaAuthoritySnapshot,
};
#[path = "root_schema/aspect_plan_quote.rs"]
mod aspect_plan_quote;
#[path = "root_schema/compilation.rs"]
mod compilation;
#[path = "root_schema/compilation_quote.rs"]
mod compilation_quote;
#[path = "root_schema/compiled_plans.rs"]
mod compiled_plans;
#[path = "root_schema/compiled_retention.rs"]
mod compiled_retention;
#[path = "root_schema/home_target.rs"]
mod home_target;
#[path = "root_schema/recorded_image.rs"]
mod recorded_image;
#[path = "root_schema/registry_binding.rs"]
mod registry_binding;
#[path = "root_schema/registry_quote.rs"]
mod registry_quote;
#[path = "root_schema/snapshot_match.rs"]
mod snapshot_match;
#[path = "root_schema/snapshot_quote.rs"]
mod snapshot_quote;
#[path = "root_schema/standard_tree_quote.rs"]
mod standard_tree_quote;
pub(crate) use compiled_retention::{
    RetainedRelationalAspectPlans, RetainedRelationalSchemaRegistry,
};
use registry_binding::registry_for_snapshot;
pub(crate) use standard_tree_quote::retained_tree_bytes as retained_compilation_tree_bytes;
#[path = "root_schema/allocation_observation.rs"]
mod allocation_observation;

/// Immutable schema vocabulary and executable projection contracts selected
/// by one branch root. Exact readers must not consult the live runtime schema.
#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRootSchemaAuthority {
    allocation_id: u64,
    authority_digest: [u8; 32],
    registry: Arc<RelationalSchemaRegistry>,
    aspect_plans: Arc<AspectContractPlanCatalog>,
    retained_aspect_contracts: Arc<Vec<AspectContract>>,
    descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    // Canonically ordered revisions already established by the original
    // snapshot admission. Sharing this fixed DATA avoids recomputing integrity
    // plans and allocating a second snapshot during exact root checks.
    relation_integrity_revisions: Arc<[crate::schema::data::RelationIntegrityPlanRevision]>,
    recorded_image: std::sync::OnceLock<crate::durability::data::DurableBranchRootSchemaImage>,
    _funding: Option<Arc<compiled_retention::SchemaCompilationFunding>>,
}

impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn capture(
        allocation_id: u64,
        registry: &RelationalSchemaRegistry,
        expected: &SchemaAuthoritySnapshot,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
        previous: Option<&Self>,
    ) -> Option<Arc<Self>> {
        registry_for_snapshot(registry, expected).map(|registry| {
            Self::compile_owned(
                allocation_id,
                registry,
                expected,
                descriptor_semantics_version,
                compilation::RetainedContracts::Extend(previous),
                None,
                crate::schema::SchemaCompilationControl::Configuration,
            )
            .expect("configuration root compilation has no request refusal")
        })
    }

    pub(crate) fn readmit_exact(
        allocation_id: u64,
        registry: RelationalSchemaRegistry,
        retained_aspect_contracts: Vec<AspectContract>,
        expected: &SchemaAuthoritySnapshot,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    ) -> Option<Arc<Self>> {
        (registry.authority_snapshot() == *expected).then(|| {
            Self::compile_owned(
                allocation_id,
                registry,
                expected,
                descriptor_semantics_version,
                compilation::RetainedContracts::Exact(retained_aspect_contracts),
                None,
                crate::schema::SchemaCompilationControl::Configuration,
            )
            .expect("configuration root compilation has no request refusal")
        })
    }

    #[cfg(test)]
    pub(crate) fn empty() -> Arc<Self> {
        let registry = RelationalSchemaRegistry::default();
        Self::compile_owned(
            0,
            registry.clone(),
            &registry.authority_snapshot(),
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
            compilation::RetainedContracts::Exact(Vec::new()),
            None,
            crate::schema::SchemaCompilationControl::Configuration,
        )
        .expect("configuration root compilation has no request refusal")
    }

    pub(crate) fn matches(&self, expected: &SchemaAuthoritySnapshot) -> bool {
        snapshot_match::matches(self, expected)
            && plans_match_snapshot(&self.aspect_plans, expected)
    }

    pub(crate) fn registry(&self) -> &RelationalSchemaRegistry {
        &self.registry
    }

    pub(crate) fn registry_arc(&self) -> RetainedRelationalSchemaRegistry {
        RetainedRelationalSchemaRegistry::from_schema(
            Arc::clone(&self.registry),
            self._funding.clone(),
        )
    }

    pub(crate) fn has_same_executable_meaning_except_basis(
        &self,
        target: &RelationalSchemaRegistry,
    ) -> bool {
        super::schema_meaning::registry_meaning_matches(&self.registry, target)
    }

    pub(crate) const fn authority_digest(&self) -> [u8; 32] {
        self.authority_digest
    }

    pub(crate) fn aspect_plans(&self) -> &AspectContractPlanCatalog {
        &self.aspect_plans
    }

    pub(crate) fn aspect_plans_arc(&self) -> RetainedRelationalAspectPlans {
        RetainedRelationalAspectPlans::from_schema(
            Arc::clone(&self.aspect_plans),
            self._funding.clone(),
        )
    }

    pub(crate) fn retained_aspect_contracts(&self) -> &[AspectContract] {
        &self.retained_aspect_contracts
    }

    pub(crate) const fn allocation_id(&self) -> u64 {
        self.allocation_id
    }

    pub(crate) fn schema_version(&self) -> crate::schema::data::SchemaVersionId {
        self.registry
            .authority_snapshot()
            .primary_schema_version_id
            .unwrap_or(crate::schema::data::SchemaVersionId(0))
    }

    pub(crate) const fn descriptor_semantics_version(
        &self,
    ) -> crate::schema::data::DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    pub(crate) fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.entity_plans.get(&kind_id)
    }

    pub(crate) fn entity_aspect_plan_count(&self) -> usize {
        self.aspect_plans.entity_plans.len()
    }

    pub(crate) fn relation_aspect_plan_count(&self) -> usize {
        self.aspect_plans.relation_plans.len()
    }

    pub(crate) fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.relation_plans.get(&kind_id)
    }
}

impl PortableAspectContractLookup for RelationalBranchRootSchemaAuthority {
    fn contract_for(&self, key: &AspectKey) -> Option<AspectContract> {
        self.retained_aspect_contracts
            .iter()
            .rev()
            .find(|contract| contract.key() == key)
            .cloned()
    }

    fn exact_contract_for(
        &self,
        key: &AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    ) -> Option<AspectContract> {
        self.retained_aspect_contracts
            .iter()
            .find(|contract| {
                contract.key() == key
                    && contract.identity() == identity
                    && contract.revision() == revision
            })
            .cloned()
    }
}

fn retain_aspect_contracts(
    previous: Option<&RelationalBranchRootSchemaAuthority>,
    plans: &AspectContractPlanCatalog,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<Vec<AspectContract>, crate::execution::RelationalExecutionDenialCause> {
    let mut retained = std::collections::BTreeMap::new();
    if let Some(previous) = previous {
        for contract in previous.retained_aspect_contracts() {
            control.checkpoint()?;
            retained.insert(contract_basis(contract), contract.clone());
        }
    }
    for binding in plans
        .entity_plans
        .values()
        .chain(plans.relation_plans.values())
        .flat_map(|plan| &plan.executable_bindings)
    {
        control.checkpoint()?;
        retained.insert(contract_basis(&binding.contract), binding.contract.clone());
    }
    let mut contracts = Vec::with_capacity(retained.len());
    for contract in retained.into_values() {
        control.checkpoint()?;
        contracts.push(contract);
    }
    Ok(contracts)
}

fn contract_basis(
    contract: &AspectContract,
) -> (AspectKey, AspectIdentity, AspectContractRevision) {
    (
        contract.key().clone(),
        contract.identity(),
        contract.revision(),
    )
}

fn plans_match_snapshot(
    plans: &AspectContractPlanCatalog,
    expected: &SchemaAuthoritySnapshot,
) -> bool {
    plans_match_snapshot_for_compilation(
        plans,
        expected,
        crate::schema::SchemaCompilationControl::Configuration,
    )
    .expect("configuration plan comparison has no request refusal")
}
fn plans_match_snapshot_for_compilation(
    plans: &AspectContractPlanCatalog,
    expected: &SchemaAuthoritySnapshot,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
    for kind in &expected.entity_kinds {
        control.checkpoint()?;
        if !plans
            .entity_plans
            .get(&kind.kind_id)
            .is_some_and(|plan| plan.plan_revision == kind.aspect_plan_revision)
        {
            return Ok(false);
        }
    }
    for kind in &expected.relation_kinds {
        control.checkpoint()?;
        if !plans
            .relation_plans
            .get(&kind.kind_id)
            .is_some_and(|plan| plan.plan_revision == kind.aspect_plan_revision)
        {
            return Ok(false);
        }
    }
    Ok(true)
}
