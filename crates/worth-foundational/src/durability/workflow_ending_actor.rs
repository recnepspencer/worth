//! Recorded workflow ending provenance, never execution or cancellation authority.
use super::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};
use crate::boundary_protocol::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolUnsupportedVersion,
    BoundaryProtocolVersion,
};
use crate::values::{EntityId, InlineBytes, PartitionId};

const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
const MAGIC: &[u8; 4] = b"WQEA";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowEndingActorProvenance {
    /// The admitted principal's coordinates in the record's Native/Home scope.
    /// These coordinates alone are not a globally scoped identity or a grant.
    Principal(EntityId),
    HomeWorldOwner(DurableWorldOwnerIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowEndingActorDecodeDenial {
    Malformed,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    UnknownActorKind(u8),
}

impl WorkflowEndingActorProvenance {
    /// One concrete encoding: exact version, kind and one complete payload.
    /// The Native scalar/frame owns physical integrity and persistence.
    pub fn recorded_bytes(self) -> InlineBytes {
        let mut bytes = Vec::with_capacity(33);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&VERSION.get().to_le_bytes());
        match self {
            Self::Principal(entity) => {
                bytes.push(0);
                bytes.extend_from_slice(&entity.partition_id.0.to_le_bytes());
                bytes.extend_from_slice(&entity.local_slot.0.to_le_bytes());
                bytes.extend_from_slice(&entity.generation.0.to_le_bytes());
            }
            Self::HomeWorldOwner(owner) => {
                bytes.push(1);
                bytes.extend_from_slice(&owner.home().bytes());
                bytes.extend_from_slice(&owner.slot().get().to_le_bytes());
            }
        }
        InlineBytes::try_new(bytes).expect("the two fixed provenance shapes fit InlineBytes")
    }

    pub fn from_recorded_bytes(bytes: &[u8]) -> Result<Self, WorkflowEndingActorDecodeDenial> {
        use WorkflowEndingActorDecodeDenial as Denial;
        if bytes.len() < 9 || &bytes[..4] != MAGIC {
            return Err(Denial::Malformed);
        }
        let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
            bytes[4..8].try_into().expect("checked version width"),
        ))
        .map_err(|_| Denial::Malformed)?;
        BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
            .admit(version)
            .map_err(Denial::UnsupportedVersion)?;
        match bytes[8] {
            0 => {
                let payload: &[u8; 16] = bytes[9..].try_into().map_err(|_| Denial::Malformed)?;
                Ok(Self::Principal(EntityId::new(
                    PartitionId(u32::from_le_bytes(payload[..4].try_into().unwrap())),
                    u64::from_le_bytes(payload[4..12].try_into().unwrap()),
                    u32::from_le_bytes(payload[12..].try_into().unwrap()),
                )))
            }
            1 => {
                let payload: &[u8; 24] = bytes[9..].try_into().map_err(|_| Denial::Malformed)?;
                let home = HomeIdentity::from_observed_bytes(payload[..16].try_into().unwrap())
                    .map_err(|_| Denial::Malformed)?;
                Ok(Self::HomeWorldOwner(
                    DurableWorldOwnerIdentity::from_recorded(
                        home,
                        WorldOwnerSlotIdentity::from_recorded(u64::from_le_bytes(
                            payload[16..].try_into().unwrap(),
                        )),
                    ),
                ))
            }
            kind => Err(Denial::UnknownActorKind(kind)),
        }
    }
}

#[cfg(test)]
mod tests;
