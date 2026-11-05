//! Diagnostic retained-allocation observations, separate from before-growth quotes.
use super::*;

impl RelationalBranchRootSchemaAuthority {
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
        .saturating_add(
            (self.relation_integrity_revisions.len() as u64).saturating_mul(std::mem::size_of::<
                crate::schema::data::RelationIntegrityPlanRevision,
            >() as u64),
        )
    }
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
