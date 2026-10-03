//! Checkpoint bytes: owned command records, or sealed certificate frames that
//! keep their capture envelope alive through command and retry ownership.

use crate::physical_runtime::durability::CheckpointCertificateFrame;

pub(in crate::physical_runtime) enum PhysicalCheckpointCommandPayload {
    Owned(Box<[u8]>),
    Certificate(CheckpointCertificateFrame),
}

impl std::ops::Deref for PhysicalCheckpointCommandPayload {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Certificate(frame) => frame.bytes(),
        }
    }
}

impl From<Box<[u8]>> for PhysicalCheckpointCommandPayload {
    fn from(bytes: Box<[u8]>) -> Self {
        Self::Owned(bytes)
    }
}
