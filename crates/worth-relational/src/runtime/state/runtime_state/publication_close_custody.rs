//! Read-only identities of the original publication holders refusing close.
use crate::mvcc::PublicationCompanionRegistrationStop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationRegistryCustody {
    Empty,
    Registered { count: usize, first_identity: u64 },
    ContactHeld,
}

impl RelationalPublicationRegistryCustody {
    pub(super) fn observed(count: usize, first_identity: Option<u64>) -> Self {
        if count == 0 {
            Self::Empty
        } else {
            Self::Registered {
                count,
                first_identity: first_identity.expect("a locked nonempty registry has a key"),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationCompanionCustody {
    Released,
    Client { generation: u64 },
    Unavailable(PublicationCompanionRegistrationStop),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalPublicationCustodyHolders {
    pub candidates: RelationalPublicationRegistryCustody,
    pub settlements: RelationalPublicationRegistryCustody,
    pub companion: RelationalPublicationCompanionCustody,
}

impl RelationalPublicationCustodyHolders {
    pub(super) fn retained(self) -> Option<Self> {
        if self.candidates == RelationalPublicationRegistryCustody::Empty
            && self.settlements == RelationalPublicationRegistryCustody::Empty
            && self.companion == RelationalPublicationCompanionCustody::Released
        {
            None
        } else {
            Some(self)
        }
    }

    /// Preserve memory close's nonwaiting, first-established-holder semantics.
    pub(super) fn memory_result(self) -> Result<Option<Self>, ()> {
        use RelationalPublicationRegistryCustody::{ContactHeld, Empty, Registered};
        match self.candidates {
            ContactHeld => Err(()),
            Registered { .. } => Ok(Some(self)),
            Empty => match self.settlements {
                ContactHeld => Err(()),
                Registered { .. } => Ok(Some(self)),
                Empty => match self.companion {
                    RelationalPublicationCompanionCustody::Released => Ok(None),
                    RelationalPublicationCompanionCustody::Client { .. } => Ok(Some(self)),
                    RelationalPublicationCompanionCustody::Unavailable(_) => Err(()),
                },
            },
        }
    }
}
