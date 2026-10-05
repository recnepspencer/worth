use std::sync::Arc;

use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, PortableAspectContractLookup,
};

use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
    SchemaAuthoritySnapshot,
};

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
            let aspect_plans = crate::schema::lower_aspect_plans(&registry);
            let retained_aspect_contracts = retain_aspect_contracts(previous, &aspect_plans);
            debug_assert!(plans_match_snapshot(&aspect_plans, expected));
            Arc::new(Self {
                allocation_id,
                authority_digest: crate::schema::data::schema_authority_snapshot_digest_bytes(
                    expected,
                ),
                registry: Arc::new(registry),
                aspect_plans: Arc::new(aspect_plans),
                retained_aspect_contracts: Arc::new(retained_aspect_contracts),
                descriptor_semantics_version,
            })
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
            let aspect_plans = crate::schema::lower_aspect_plans(&registry);
            debug_assert!(plans_match_snapshot(&aspect_plans, expected));
            Arc::new(Self {
                allocation_id,
                authority_digest: crate::schema::data::schema_authority_snapshot_digest_bytes(
                    expected,
                ),
                registry: Arc::new(registry),
                aspect_plans: Arc::new(aspect_plans),
                retained_aspect_contracts: Arc::new(retained_aspect_contracts),
                descriptor_semantics_version,
            })
        })
    }

    #[cfg(test)]
    pub(crate) fn empty() -> Arc<Self> {
        Arc::new(Self {
            allocation_id: 0,
            authority_digest: RelationalSchemaRegistry::default().authority_digest_bytes(),
            registry: Arc::new(RelationalSchemaRegistry::default()),
            aspect_plans: Arc::new(AspectContractPlanCatalog::empty()),
            retained_aspect_contracts: Arc::new(Vec::new()),
            descriptor_semantics_version: crate::schema::data::runtime_descriptor_semantics_policy(
            )
            .current_write_version(),
        })
    }

    pub(crate) fn matches(&self, expected: &SchemaAuthoritySnapshot) -> bool {
        self.registry.authority_snapshot() == *expected
            && plans_match_snapshot(&self.aspect_plans, expected)
    }

    pub(crate) fn registry(&self) -> &RelationalSchemaRegistry {
        &self.registry
    }

    pub(crate) fn registry_arc(&self) -> Arc<RelationalSchemaRegistry> {
        Arc::clone(&self.registry)
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

    pub(crate) fn retained_aspect_contracts(&self) -> &[AspectContract] {
        &self.retained_aspect_contracts
    }

    pub(crate) const fn allocation_id(&self) -> u64 {
        self.allocation_id
    }

    pub(crate) fn authoritative_allocation_bytes(&self) -> u64 {
        let registry_bytes = registry_allocation_bytes(&self.registry);
        let plan_bytes = plan_catalog_allocation_bytes(&self.aspect_plans);
        let retained_contract_bytes = self.retained_aspect_contracts.iter().fold(
            self.retained_aspect_contracts
                .capacity()
                .saturating_mul(std::mem::size_of::<AspectContract>()),
            |bytes, contract| bytes.saturating_add(contract.owned_allocation_capacity_bytes()),
        );
        usize_to_u64(
            std::mem::size_of::<Self>()
                .saturating_add(registry_bytes)
                .saturating_add(plan_bytes)
                .saturating_add(retained_contract_bytes),
        )
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
) -> Vec<AspectContract> {
    let mut retained = std::collections::BTreeMap::new();
    if let Some(previous) = previous {
        for contract in previous.retained_aspect_contracts() {
            retained.insert(contract_basis(contract), contract.clone());
        }
    }
    for binding in plans
        .entity_plans
        .values()
        .chain(plans.relation_plans.values())
        .flat_map(|plan| &plan.executable_bindings)
    {
        retained.insert(contract_basis(&binding.contract), binding.contract.clone());
    }
    retained.into_values().collect()
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

fn registry_allocation_bytes(registry: &RelationalSchemaRegistry) -> usize {
    let entity_bytes = registry.entity_kinds.values().fold(0_usize, |bytes, kind| {
        bytes
            .saturating_add(std::mem::size_of_val(kind))
            .saturating_add(kind.kind_name.capacity())
            .saturating_add(kind.schema_id.0.capacity())
            .saturating_add(declaration_allocation_bytes(
                &kind.aspect_contract_declarations,
            ))
    });
    let relation_bytes = registry
        .relation_kinds
        .values()
        .fold(0_usize, |bytes, kind| {
            bytes
                .saturating_add(std::mem::size_of_val(kind))
                .saturating_add(kind.kind_name.capacity())
                .saturating_add(kind.schema_id.0.capacity())
                .saturating_add(declaration_allocation_bytes(
                    &kind.aspect_contract_declarations,
                ))
        });
    std::mem::size_of::<RelationalSchemaRegistry>()
        .saturating_add(entity_bytes)
        .saturating_add(relation_bytes)
}

fn declaration_allocation_bytes(
    declarations: &crate::schema::data::KindAspectContractDeclarations,
) -> usize {
    declarations
        .aspects
        .iter()
        .fold(
            declarations
                .aspects
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    crate::schema::data::DeclaredAspectContractBinding,
                >()),
            |bytes, declaration| {
                bytes
                    .saturating_add(declaration.binding.owned_allocation_capacity_bytes())
                    .saturating_add(declaration.contract.owned_allocation_capacity_bytes())
            },
        )
        .saturating_add(
            declarations
                .identity_declarations
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    crate::merge::data::IdentityBasisDeclaration,
                >()),
        )
        .saturating_add(
            declarations
                .merge_policy_declarations
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    crate::merge::data::AspectMergePolicyDeclaration,
                >()),
        )
}

fn plan_catalog_allocation_bytes(plans: &AspectContractPlanCatalog) -> usize {
    let bindings = plans
        .entity_plans
        .values()
        .chain(plans.relation_plans.values())
        .fold(0_usize, |bytes, plan| {
            plan.executable_bindings.iter().fold(
                bytes
                    .saturating_add(std::mem::size_of_val(plan))
                    .saturating_add(plan.executable_bindings.capacity().saturating_mul(
                        std::mem::size_of::<crate::schema::data::LoweredAspectContractBinding>(),
                    )),
                |bytes, binding| {
                    bytes
                        .saturating_add(binding.target.owned_allocation_capacity_bytes())
                        .saturating_add(binding.contract.owned_allocation_capacity_bytes())
                },
            )
        });
    std::mem::size_of::<AspectContractPlanCatalog>().saturating_add(bindings)
}

fn usize_to_u64(bytes: usize) -> u64 {
    u64::try_from(bytes).unwrap_or(u64::MAX)
}

/// Re-admit an older basis only when the available registry carries exactly
/// the same executable kind meaning. A changed schema id/version may be
/// rebound from the envelope; changed names, plans, integrity, or policies
/// require their own historical registry and are denied here.
fn registry_for_snapshot(
    available: &RelationalSchemaRegistry,
    expected: &SchemaAuthoritySnapshot,
) -> Option<RelationalSchemaRegistry> {
    if available.authority_snapshot() == *expected {
        return Some(available.clone());
    }
    let observed = available.authority_snapshot();
    let entity_meaning_matches = observed.entity_kinds.len() == expected.entity_kinds.len()
        && observed
            .entity_kinds
            .iter()
            .zip(&expected.entity_kinds)
            .all(|(observed, expected)| {
                observed.kind_id == expected.kind_id
                    && observed.kind_name == expected.kind_name
                    && observed.aspect_plan_revision == expected.aspect_plan_revision
            });
    let relation_meaning_matches = observed.relation_kinds.len() == expected.relation_kinds.len()
        && observed
            .relation_kinds
            .iter()
            .zip(&expected.relation_kinds)
            .all(|(observed, expected)| {
                observed.kind_id == expected.kind_id
                    && observed.kind_name == expected.kind_name
                    && observed.aspect_plan_revision == expected.aspect_plan_revision
                    && observed.relation_integrity_plan_revision
                        == expected.relation_integrity_plan_revision
                    && observed.cross_context_policy == expected.cross_context_policy
                    && observed.cascade_delete_policy == expected.cascade_delete_policy
            });
    if !entity_meaning_matches || !relation_meaning_matches {
        return None;
    }

    let mut rebound = available.clone();
    for expected_kind in &expected.entity_kinds {
        let registration = rebound.entity_kinds.get_mut(&expected_kind.kind_id)?;
        registration.schema_id = expected_kind.schema_id.clone();
        registration.schema_version_id = expected_kind.schema_version_id;
    }
    for expected_kind in &expected.relation_kinds {
        let registration = rebound.relation_kinds.get_mut(&expected_kind.kind_id)?;
        registration.schema_id = expected_kind.schema_id.clone();
        registration.schema_version_id = expected_kind.schema_version_id;
    }
    (rebound.authority_snapshot() == *expected).then_some(rebound)
}

fn plans_match_snapshot(
    plans: &AspectContractPlanCatalog,
    expected: &SchemaAuthoritySnapshot,
) -> bool {
    expected.entity_kinds.iter().all(|kind| {
        plans
            .entity_plans
            .get(&kind.kind_id)
            .is_some_and(|plan| plan.plan_revision == kind.aspect_plan_revision)
    }) && expected.relation_kinds.iter().all(|kind| {
        plans
            .relation_plans
            .get(&kind.kind_id)
            .is_some_and(|plan| plan.plan_revision == kind.aspect_plan_revision)
    })
}
