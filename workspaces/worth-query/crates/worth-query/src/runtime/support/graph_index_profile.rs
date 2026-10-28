use super::profile::WorthQueryRuntimeSupportProfile;
use crate::runtime::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphIndexInventory,
    WorthQueryGraphIndexLifecycleClass, WorthQueryGraphIndexLifecycleOwner,
    WorthQueryGraphIndexPosture, WorthQueryGraphIndexSupportRow, WorthQueryGraphIndexSupportState,
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessRebuildBasis, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadLifecycleClass, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPredicateFamily,
};

impl WorthQueryRuntimeSupportProfile {
    pub fn graph_index_inventory(&self) -> WorthQueryGraphIndexInventory {
        WorthQueryGraphIndexInventory::from_rows(self.graph_index_support_rows.clone())
    }

    pub fn with_graph_index_support_omitted(
        mut self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    ) -> Self {
        self.graph_index_support_rows
            .retain(|row| row.requirement_kind() != &requirement_kind);
        self
    }

    pub fn with_graph_index_access_capability_registration_required(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        owning_milestone: impl Into<String>,
    ) -> Self {
        self.with_graph_index_support_posture(
            requirement_kind,
            WorthQueryGraphIndexLifecycleOwner::DomainRegistration,
            WorthQueryGraphIndexLifecycleClass::AccessCapabilityRegistrationRequired,
            WorthQueryGraphIndexPosture::RequiresAccessCapabilityRegistration,
            WorthQueryGraphIndexSupportState::Declared,
            Some(owning_milestone.into()),
        )
    }

    pub fn with_graph_index_temporarily_unavailable(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    ) -> Self {
        self.with_graph_index_support_posture(
            requirement_kind,
            WorthQueryGraphIndexLifecycleOwner::LowerRuntime,
            WorthQueryGraphIndexLifecycleClass::TemporarilyUnavailable,
            WorthQueryGraphIndexPosture::TemporarilyUnavailable,
            WorthQueryGraphIndexSupportState::TemporarilyUnavailable,
            None,
        )
    }

    pub fn with_graph_index_ephemeral_available(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    ) -> Self {
        self.with_graph_index_support_posture(
            requirement_kind,
            WorthQueryGraphIndexLifecycleOwner::QueryRuntime,
            WorthQueryGraphIndexLifecycleClass::EphemeralRuntimeOwned,
            WorthQueryGraphIndexPosture::EphemeralAvailable,
            WorthQueryGraphIndexSupportState::Available,
            None,
        )
    }

    pub fn with_store_backed_graph_index_requirement(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        owning_milestone: impl Into<String>,
    ) -> Self {
        self.with_graph_index_support_posture(
            requirement_kind,
            WorthQueryGraphIndexLifecycleOwner::StoreOwned,
            WorthQueryGraphIndexLifecycleClass::StoreOwnedRequired,
            WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex,
            WorthQueryGraphIndexSupportState::StoreOwnedUnavailable,
            Some(owning_milestone.into()),
        )
    }

    pub fn with_graph_index_supported_relation_direction(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        direction: WorthQueryAdmittedGraphReadRelationDirection,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_supported_relation_direction(direction.clone())
        })
    }

    pub fn with_graph_index_supported_predicate_family(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        predicate_family: WorthQueryGraphReadPredicateFamily,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_supported_predicate_family(predicate_family.clone())
        })
    }

    pub fn with_graph_index_supported_ordering_posture(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        ordering_posture: WorthQueryGraphReadOrderingPosture,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_supported_ordering_posture(ordering_posture.clone())
        })
    }

    pub fn with_graph_index_supported_requirement_lifecycle(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        lifecycle: WorthQueryGraphReadLifecycleClass,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_supported_requirement_lifecycle(lifecycle.clone())
        })
    }

    pub fn with_graph_index_rebuild_basis(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_rebuild_basis(rebuild_basis.clone())
        })
    }

    pub fn with_graph_index_invalidation_basis(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_invalidation_basis(invalidation_basis.clone())
        })
    }

    pub fn with_graph_index_complexity_contract(
        self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        complexity_contract: WorthQueryGraphReadAccessComplexityContract,
    ) -> Self {
        self.transform_graph_index_support_row(requirement_kind, |row| {
            row.with_complexity_contract(complexity_contract.clone())
        })
    }

    fn with_graph_index_support_posture(
        mut self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        lifecycle_owner: WorthQueryGraphIndexLifecycleOwner,
        lifecycle_class: WorthQueryGraphIndexLifecycleClass,
        posture: WorthQueryGraphIndexPosture,
        support_state: WorthQueryGraphIndexSupportState,
        owning_milestone: Option<String>,
    ) -> Self {
        self.graph_index_support_rows
            .retain(|row| row.requirement_kind() != &requirement_kind);
        for profile in WorthQueryGraphIndexInventory::current_runtime_support_rows()
            .into_iter()
            .filter(|row| row.requirement_kind() == &requirement_kind)
        {
            self.graph_index_support_rows.push(
                WorthQueryGraphIndexSupportRow::with_runtime_support_posture(
                    requirement_kind.clone(),
                    lifecycle_owner.clone(),
                    lifecycle_class.clone(),
                    posture.clone(),
                    support_state.clone(),
                    owning_milestone.clone(),
                )
                .with_complexity_contract(profile.complexity_contract().clone()),
            );
        }
        self
    }

    fn transform_graph_index_support_row(
        mut self,
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        mut transform: impl FnMut(WorthQueryGraphIndexSupportRow) -> WorthQueryGraphIndexSupportRow,
    ) -> Self {
        for row in &mut self.graph_index_support_rows {
            if row.requirement_kind() == &requirement_kind {
                *row = transform(row.clone());
            }
        }
        self
    }
}

#[cfg(test)]
mod tests;
