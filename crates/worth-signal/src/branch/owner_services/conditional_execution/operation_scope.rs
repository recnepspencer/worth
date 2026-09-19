use crate::branch::{AdmittedSignalBranchBasis, SignalBranchBasisAdmissionIdentity};
use crate::data::graph::SignalGraph;
use crate::state::SignalBranchId;

use super::{SignalConditionalExecutionPort, SignalInstalledDefinitionBinding};
use crate::branch::owner_services::SignalBranchCellIncarnation;

/// Private proof that a canonical owner transaction already holds one exact
/// branch cell. It authorizes nested conditional evaluation only; it grants no
/// publication, mutation, or graph capability of its own.
pub(crate) struct SignalConditionalOperationScopeBinding {
    basis_admission_identity: SignalBranchBasisAdmissionIdentity,
    branch_id: SignalBranchId,
    incarnation: SignalBranchCellIncarnation,
    definition: Option<SignalInstalledDefinitionBinding>,
    definition_publication: Option<SignalConditionalDefinitionPublicationScope>,
}

#[derive(Clone)]
pub(crate) struct SignalConditionalDefinitionPublicationScope {
    pub(super) authority: std::sync::Arc<super::SignalConditionalServiceAuthority>,
    ordinal: u64,
}

impl SignalConditionalDefinitionPublicationScope {
    pub(super) fn issue(
        authority: std::sync::Arc<super::SignalConditionalServiceAuthority>,
        ordinal: u64,
    ) -> Self {
        Self { authority, ordinal }
    }

    pub(super) fn matches(&self, candidate: &Self) -> bool {
        self.ordinal == candidate.ordinal
            && std::sync::Arc::ptr_eq(&self.authority, &candidate.authority)
    }
}

impl SignalConditionalOperationScopeBinding {
    pub(in crate::branch::owner_services) fn issue(
        basis: &AdmittedSignalBranchBasis,
        incarnation: SignalBranchCellIncarnation,
        definition: Option<SignalInstalledDefinitionBinding>,
    ) -> Self {
        Self {
            basis_admission_identity: basis.admission_identity().clone(),
            branch_id: basis.branch_id(),
            incarnation,
            definition,
            definition_publication: None,
        }
    }

    pub(in crate::branch::owner_services) fn with_definition_publication(
        mut self,
        publication: SignalConditionalDefinitionPublicationScope,
    ) -> Self {
        self.definition_publication = Some(publication);
        self
    }

    pub(crate) fn admits_port<D, I, T>(
        &self,
        port: &SignalConditionalExecutionPort<D, I, T>,
        graph: &SignalGraph,
    ) -> bool
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Copy + Ord,
    {
        self.basis_admission_identity == *port.basis.admission_identity()
            && self.branch_id == port.basis.branch_id()
            && self.branch_id == graph.current_branch().id
            && self.incarnation == port.incarnation
            && self
                .definition
                .as_ref()
                .is_some_and(|installed| installed.matches(&port.definition))
    }

    pub(crate) fn admits_definition_publication(
        &self,
        candidate: &SignalConditionalDefinitionPublicationScope,
    ) -> bool {
        self.definition_publication
            .as_ref()
            .is_some_and(|installed| installed.matches(candidate))
    }
}
