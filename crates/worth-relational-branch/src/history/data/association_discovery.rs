//! Required directory meaning used to select exact artifact bodies lazily.
use serde::{Deserialize, Serialize};

use super::{CanonicalCommitEnvelopeBody, RelationalCallerAssociation};

/// Discovery metadata grants no durable origin, settlement, root, or recovery
/// authority. The selected source artifact must independently readmit it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelationalAssociationDiscovery {
    Unassociated,
    Associated(RelationalCallerAssociation),
}

impl RelationalAssociationDiscovery {
    pub fn association(&self) -> Option<&RelationalCallerAssociation> {
        match self {
            Self::Unassociated => None,
            Self::Associated(association) => Some(association),
        }
    }
}

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn association_discovery(&self) -> RelationalAssociationDiscovery {
        match self.caller_association() {
            Some(association) => RelationalAssociationDiscovery::Associated(association.clone()),
            None => RelationalAssociationDiscovery::Unassociated,
        }
    }
}
