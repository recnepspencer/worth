//! Completed reads reference scopes in their actual immutable installed contract.
use std::sync::Arc;
use worth_execution::ExecutionArray;
use worth_query_installation::facade::{
    WorthQueryCompiledApplicationOperationContracts as Contracts,
    WorthQueryOperationGraphReadScope as Scope,
};

/// Descriptive position only. No source, permission or currentness grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct InstalledScopePosition {
    role: usize,
    scope: usize,
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
                    .map(|scope| Self { role, scope })
            })
    }
    pub(super) fn resolve(self, contracts: &Contracts) -> Option<&Scope> {
        contracts
            .graph_reads()
            .roles()
            .get(self.role)?
            .read_scopes()
            .get(self.scope)
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
    pub(super) fn backing_bytes(&self) -> Option<u64> {
        match self {
            Self::Installed { positions, .. } => {
                (size_of::<ExecutionArray<InstalledScopePosition>>() as u64)
                    .checked_add(
                        (positions
                            .len()
                            .checked_mul(size_of::<InstalledScopePosition>())?)
                            as u64,
                    )?
                    .checked_add(2 * size_of::<usize>() as u64)
            }
            #[cfg(test)]
            Self::EmptyOwnerFixture => Some(0),
        }
    }
}
