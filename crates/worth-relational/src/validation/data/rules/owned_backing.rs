//! Direct heap backing retained by a native rule, without encoding or cloning it.
use super::InvariantRule;
use crate::identity::data::KindId;

impl InvariantRule {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::LiveRecordRequiresSidecar(_)
            | Self::MaxMergedIntents(_)
            | Self::MaxSnapshotEntities(_) => 0,
            Self::UniqueEntityAspectField { field_locator } => {
                field_locator.owned_allocation_capacity_bytes() as u64
            }
            Self::EndpointKindContract(contract) => contract
                .contract_id
                .owned_allocation_capacity_bytes()
                .saturating_add(kind_vec(&contract.allowed_source_kinds))
                .saturating_add(kind_vec(&contract.allowed_target_kinds)),
            Self::CardinalityMaximumContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::CardinalityMinimumContract(contract) => contract
                .contract_id
                .owned_allocation_capacity_bytes()
                .saturating_add(kind_vec(&contract.candidate_source_kinds))
                .saturating_add(kind_vec(&contract.candidate_target_kinds)),
            Self::UniquenessContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::SymmetryContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::EndpointDeletionIntegrityContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::AcyclicityContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::PartitionIsolationContract(contract) => {
                contract.contract_id.owned_allocation_capacity_bytes()
            }
            Self::ConnectivityMinimumContract(contract) => contract
                .contract_id
                .owned_allocation_capacity_bytes()
                .saturating_add(kind_vec(&contract.source_kind_ids))
                .saturating_add(kind_vec(&contract.target_kind_ids)),
        }
    }
}

fn kind_vec(kinds: &Vec<KindId>) -> u64 {
    (kinds.capacity() as u64).saturating_mul(std::mem::size_of::<KindId>() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::data::{
        ConnectivityMinimumEnforcement, ContractId, LoweredConnectivityMinimumContract,
        RelationIntegrityPlanRevision,
    };

    #[test]
    fn retained_native_rule_counts_spare_contract_and_kind_backing() {
        // Allocation shape only: this value does not claim plan or graph authority.
        let mut name = String::with_capacity(173);
        name.push_str("owned-native-minimum");
        let mut sources = Vec::with_capacity(17);
        sources.push(KindId(1));
        let mut targets = Vec::with_capacity(11);
        targets.push(KindId(3));
        let actual = name.capacity()
            + (sources.capacity() + targets.capacity()) * std::mem::size_of::<KindId>();
        let rule = InvariantRule::ConnectivityMinimumContract(LoweredConnectivityMinimumContract {
            contract_id: ContractId::new(name),
            source_kind_ids: sources,
            target_kind_ids: targets,
            relation_kind_id: KindId(2),
            minimum_reachable_targets: 1,
            enforcement_boundary: ConnectivityMinimumEnforcement::SnapshotPublication,
            plan_revision: RelationIntegrityPlanRevision(0),
        });
        assert_eq!(rule.owned_allocation_capacity_bytes(), actual as u64);
        let cloned = rule.clone();
        let InvariantRule::ConnectivityMinimumContract(contract) = &cloned else {
            unreachable!()
        };
        let cloned_actual = contract.contract_id.owned_allocation_capacity_bytes()
            + kind_vec(&contract.source_kind_ids)
            + kind_vec(&contract.target_kind_ids);
        assert_eq!(cloned.owned_allocation_capacity_bytes(), cloned_actual);
        assert!(
            cloned_actual > 0,
            "native payload cannot be counted as absent"
        );
    }
}
