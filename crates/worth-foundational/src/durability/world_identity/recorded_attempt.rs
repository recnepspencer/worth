use super::{DurableCompositePublicationAttemptIdentity, WorldOwnerCoordinate};

/// Original data retains its backing class; a memory ordinal never becomes a
/// Home-scoped durable identity by passing through a recovered observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordedCompositePublicationAttemptIdentity {
    Home(DurableCompositePublicationAttemptIdentity),
    Memory { owner: u64, ordinal: u64 },
}

impl RecordedCompositePublicationAttemptIdentity {
    pub const fn ordinal(self) -> u64 {
        match self {
            Self::Home(value) => value.ordinal(),
            Self::Memory { ordinal, .. } => ordinal,
        }
    }
    pub const fn owner(self) -> WorldOwnerCoordinate {
        match self {
            Self::Home(value) => WorldOwnerCoordinate::Home {
                owner: value.owner(),
            },
            Self::Memory { owner, .. } => WorldOwnerCoordinate::MemoryOnly { observation: owner },
        }
    }
    pub const fn durable(self) -> Option<DurableCompositePublicationAttemptIdentity> {
        match self {
            Self::Home(value) => Some(value),
            Self::Memory { .. } => None,
        }
    }
}
