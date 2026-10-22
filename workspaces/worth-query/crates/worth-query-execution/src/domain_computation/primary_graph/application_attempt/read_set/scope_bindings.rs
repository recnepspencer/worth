//! Completed reads reference scopes in their actual immutable installed contract.
use std::sync::Arc;
use worth_execution::ExecutionArray;
use worth_query_installation::facade::{
    WorthQueryCompiledApplicationOperationContracts as Contracts,
    WorthQueryOperationGraphReadScope as Scope,
};

/// Descriptive position only. No source, permission or currentness grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum InstalledScopePosition {
    Physical {
        role: usize,
        scope: usize,
    },
    /// Descriptive only; the sealed native read validates its own query authority.
    SourceQuery,
}
impl InstalledScopePosition {
    pub(super) fn from_installed(contracts: &Contracts, scope: &Scope) -> Option<Self> {
        contracts
            .graph_reads()
            .roles()
            .iter()
            .enumerate()
            .find_map(|(role, value)| {
                value
                    .read_scopes()
                    .iter()
                    .position(|candidate| std::ptr::eq(candidate, scope))
                    .map(|scope| Self::Physical { role, scope })
            })
    }
    pub(super) fn resolve(self, contracts: &Contracts) -> Option<&Scope> {
        let Self::Physical { role, scope } = self else {
            return None;
        };
        contracts
            .graph_reads()
            .roles()
            .get(role)?
            .read_scopes()
            .get(scope)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum CompletedReadScopes {
    Installed {
        contracts: Arc<Contracts>,
        positions: Arc<ExecutionArray<InstalledScopePosition>>,
    },
    #[cfg(test)]
    EmptyOwnerFixture,
}
impl CompletedReadScopes {
    pub(super) fn preimage_demand(
        &self,
    ) -> Option<&worth_query_installation::facade::InstalledPreImageDemand> {
        use worth_query_installation::facade::InstalledCorrectionMechanism;
        match self {
            Self::Installed { contracts, .. } => match contracts.aftermath()?.mechanism()? {
                InstalledCorrectionMechanism::RecordedInverse(inverse) => {
                    Some(inverse.preimage_demand())
                }
                InstalledCorrectionMechanism::Compensation(_) => None,
            },
            #[cfg(test)]
            Self::EmptyOwnerFixture => None,
        }
    }
    pub(super) fn from_completed(
        contracts: Arc<Contracts>,
        positions: ExecutionArray<InstalledScopePosition>,
    ) -> Self {
        Self::Installed {
            contracts,
            positions: Arc::new(positions),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        match self {
            Self::Installed { positions, .. } => positions.len(),
            #[cfg(test)]
            Self::EmptyOwnerFixture => 0,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn get(
        &self,
        ordinal: usize,
    ) -> Option<&Scope> {
        match self {
            Self::Installed {
                contracts,
                positions,
            } => positions.get(ordinal)?.resolve(contracts),
            #[cfg(test)]
            Self::EmptyOwnerFixture => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn same_contract(
        &self,
        current: &Arc<Contracts>,
    ) -> bool {
        match self {
            Self::Installed { contracts, .. } => Arc::ptr_eq(contracts, current),
            #[cfg(test)]
            Self::EmptyOwnerFixture => false,
        }
    }
    pub(super) fn position(&self, ordinal: usize) -> Option<InstalledScopePosition> {
        match self {
            Self::Installed { positions, .. } => positions.get(ordinal).copied(),
            #[cfg(test)]
            Self::EmptyOwnerFixture => None,
        }
    }
}
