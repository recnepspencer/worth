use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::graph::SignalGraph;

/// Live custody of installed definition meaning, retained with branch state.
/// This metadata is not cell admission or permission to execute a conditional.
/// Exact basis/incarnation admission remains required at service entry.
#[derive(Debug, Clone)]
pub(crate) struct SignalInstalledDefinitionBinding {
    definition_basis: u64,
    claimant: Option<SignalAspectLoweringOwner>,
}

impl SignalInstalledDefinitionBinding {
    pub(crate) fn capture(graph: &SignalGraph, definition_basis: u64) -> Self {
        Self {
            definition_basis,
            claimant: graph.aspect_lowering_owner.clone(),
        }
    }

    pub(crate) fn definition_basis(&self) -> u64 {
        self.definition_basis
    }

    pub(crate) fn is_claimed_by(&self, claimant: &SignalAspectLoweringOwner) -> bool {
        self.claimant
            .as_ref()
            .is_some_and(|installed| installed.is_same_owner(claimant))
    }

    pub(crate) fn matches(&self, candidate: &Self) -> bool {
        self.definition_basis == candidate.definition_basis
            && match (&self.claimant, &candidate.claimant) {
                (Some(current), Some(candidate)) => current.is_same_owner(candidate),
                (None, None) => true,
                _ => false,
            }
    }
}
