//! Endpoint-domain meaning with the physical scalar grammar preserved.
use crate::identity::data::KindId;
use crate::internal_port::projected_member_domain::definition::ProjectedMemberDomainIdentityOwnerPort as _;
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use serde::Serialize;
pub mod decoding;

/// The domain of a graph endpoint, never a registered source handle.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NativeMemberDomain {
    Entity(KindId),
    Projected {
        projected: ProjectedMemberDomainIdentity,
    },
}

impl Serialize for NativeMemberDomain {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Entity(kind) => kind.serialize(serializer),
            Self::Projected { projected } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("projected", projected)?;
                map.end()
            }
        }
    }
}

impl From<KindId> for NativeMemberDomain {
    fn from(kind: KindId) -> Self {
        Self::Entity(kind)
    }
}

impl NativeMemberDomain {
    pub const fn entity_kind(&self) -> Option<KindId> {
        match self {
            Self::Entity(kind) => Some(*kind),
            Self::Projected { .. } => None,
        }
    }
}

/// Operations used by the Relational owners.
pub trait NativeMemberDomainOwnerPort: Sized {
    fn name_bytes(&self) -> Option<usize>;
    fn owned_bytes(&self) -> usize;
}

impl NativeMemberDomainOwnerPort for NativeMemberDomain {
    fn name_bytes(&self) -> Option<usize> {
        match self {
            Self::Entity(_) => Some(0),
            Self::Projected { projected } => projected.name_bytes(),
        }
    }

    fn owned_bytes(&self) -> usize {
        match self {
            Self::Entity(_) => 0,
            Self::Projected { projected } => projected.owned_bytes(),
        }
    }
}
