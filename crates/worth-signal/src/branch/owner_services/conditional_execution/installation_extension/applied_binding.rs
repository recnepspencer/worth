use std::sync::Arc;

use crate::branch::AdmittedSignalBranchBasis;

use super::super::{
    SignalConditionalDefinitionPublicationScope, SignalConditionalServiceAuthority,
};
use crate::branch::owner_services::SignalBranchCellIncarnation;

/// Signal-minted evidence that one exact definition publication advanced its
/// admitted predecessor. It is move-only and can complete only the paired
/// installation request.
pub struct SignalConditionalDefinitionAdvanceBinding {
    service_authority: Arc<SignalConditionalServiceAuthority>,
    publication_scope: SignalConditionalDefinitionPublicationScope,
    predecessor: AdmittedSignalBranchBasis,
    incarnation: SignalBranchCellIncarnation,
    successor: AdmittedSignalBranchBasis,
}

impl std::fmt::Debug for SignalConditionalDefinitionAdvanceBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalConditionalDefinitionAdvanceBinding")
            .field("predecessor", &self.predecessor)
            .field("successor", &self.successor)
            .finish_non_exhaustive()
    }
}

pub(in crate::branch::owner_services) struct SignalConditionalDefinitionAdvanceMint {
    service_authority: Arc<SignalConditionalServiceAuthority>,
    publication_scope: SignalConditionalDefinitionPublicationScope,
    predecessor: AdmittedSignalBranchBasis,
    incarnation: SignalBranchCellIncarnation,
}

impl SignalConditionalDefinitionAdvanceMint {
    pub(super) fn new(
        service_authority: Arc<SignalConditionalServiceAuthority>,
        publication_scope: SignalConditionalDefinitionPublicationScope,
        predecessor: AdmittedSignalBranchBasis,
        incarnation: SignalBranchCellIncarnation,
    ) -> Self {
        Self {
            service_authority,
            publication_scope,
            predecessor,
            incarnation,
        }
    }

    pub(in crate::branch::owner_services) fn bind(
        self,
        successor: AdmittedSignalBranchBasis,
    ) -> SignalConditionalDefinitionAdvanceBinding {
        SignalConditionalDefinitionAdvanceBinding {
            service_authority: self.service_authority,
            publication_scope: self.publication_scope,
            predecessor: self.predecessor,
            incarnation: self.incarnation,
            successor,
        }
    }
}

impl SignalConditionalDefinitionAdvanceBinding {
    pub(super) fn into_successor_if_matches(
        self,
        service_authority: &Arc<SignalConditionalServiceAuthority>,
        publication_scope: &SignalConditionalDefinitionPublicationScope,
        predecessor: &AdmittedSignalBranchBasis,
        incarnation: SignalBranchCellIncarnation,
    ) -> Option<AdmittedSignalBranchBasis> {
        (Arc::ptr_eq(&self.service_authority, service_authority)
            && self.publication_scope.matches(publication_scope)
            && self.predecessor.admission_identity() == predecessor.admission_identity()
            && self.predecessor.observation() == predecessor.observation()
            && self.incarnation == incarnation)
            .then_some(self.successor)
    }
}
