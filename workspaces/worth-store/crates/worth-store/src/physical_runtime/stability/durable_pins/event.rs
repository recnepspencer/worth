//! Constant-size pin journal entries.
//!
//! Each pin transition appends one entry naming the resulting state, so open
//! replays the journal in order through the same kernel transitions. Layout,
//! little-endian:
//!
//! ```text
//! "WRPN" | version u8 = 1 | kind u8 | pin u64
//! Acquire only: root u64 | policy u8 | parent flag u8, parent u64
//!               | tail flag u8, segment u64, generation u64 | payload len u16, payload
//! ```
use crate::pin_table::{RootPinDropPolicy, RootPinId};

const MAGIC: &[u8; 4] = b"WRPN";
const VERSION: u8 = 1;
/// Owner payload ceiling; one pin record stays constant-size.
pub(in crate::physical_runtime) const MAXIMUM_PIN_PAYLOAD_BYTES: usize = 512;
pub(in crate::physical_runtime) const MAXIMUM_PIN_ENTRY_BYTES: u64 =
    (14 + 8 + 1 + 9 + 17 + 2 + MAXIMUM_PIN_PAYLOAD_BYTES) as u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RootPinEvent {
    Acquire {
        pin: RootPinId,
        root: u64,
        policy: RootPinDropPolicy,
        parent: Option<RootPinId>,
        inline_tail: Option<(u64, u64)>,
        payload: Vec<u8>,
    },
    Expiring {
        pin: RootPinId,
    },
    Expired {
        pin: RootPinId,
    },
}

impl RootPinEvent {
    pub(in crate::physical_runtime) fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(MAGIC);
        bytes.push(VERSION);
        match self {
            Self::Acquire {
                pin,
                root,
                policy,
                parent,
                inline_tail,
                payload,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(&pin.0.to_le_bytes());
                bytes.extend_from_slice(&root.to_le_bytes());
                bytes.push(match policy {
                    RootPinDropPolicy::RefuseIfPinnedByChild => 0,
                    RootPinDropPolicy::RetainUntilChildrenDrain => 1,
                });
                bytes.push(u8::from(parent.is_some()));
                bytes.extend_from_slice(&parent.map_or(0, |parent| parent.0).to_le_bytes());
                let (segment, generation) = inline_tail.unwrap_or((0, 0));
                bytes.push(u8::from(inline_tail.is_some()));
                bytes.extend_from_slice(&segment.to_le_bytes());
                bytes.extend_from_slice(&generation.to_le_bytes());
                let length = u16::try_from(payload.len()).expect("payload length is bounded");
                bytes.extend_from_slice(&length.to_le_bytes());
                bytes.extend_from_slice(payload);
            }
            Self::Expiring { pin } => {
                bytes.push(2);
                bytes.extend_from_slice(&pin.0.to_le_bytes());
            }
            Self::Expired { pin } => {
                bytes.push(3);
                bytes.extend_from_slice(&pin.0.to_le_bytes());
            }
        }
        bytes
    }

    /// Decode one entry; any malformed byte is damage.
    pub(in crate::physical_runtime) fn decode(bytes: &[u8]) -> Option<Self> {
        let mut cursor = Cursor { bytes, at: 0 };
        if cursor.take(4)? != MAGIC || cursor.u8()? != VERSION {
            return None;
        }
        let kind = cursor.u8()?;
        let pin = RootPinId(cursor.u64()?);
        let event = match kind {
            1 => {
                let root = cursor.u64()?;
                let policy = match cursor.u8()? {
                    0 => RootPinDropPolicy::RefuseIfPinnedByChild,
                    1 => RootPinDropPolicy::RetainUntilChildrenDrain,
                    _ => return None,
                };
                let parent = cursor.flagged(|cursor| cursor.u64().map(RootPinId))?;
                let inline_tail = cursor.flagged(|cursor| Some((cursor.u64()?, cursor.u64()?)))?;
                let length = usize::from(u16::from_le_bytes(cursor.take(2)?.try_into().ok()?));
                if length > MAXIMUM_PIN_PAYLOAD_BYTES {
                    return None;
                }
                let payload = cursor.take(length)?.to_vec();
                Self::Acquire {
                    pin,
                    root,
                    policy,
                    parent,
                    inline_tail,
                    payload,
                }
            }
            2 => Self::Expiring { pin },
            3 => Self::Expired { pin },
            _ => return None,
        };
        (cursor.at == bytes.len()).then_some(event)
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(count)?;
        let slice = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(slice)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|bytes| bytes[0])
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    /// A flag byte followed by a fixed-width value that is zero when absent.
    fn flagged<T>(&mut self, read: impl FnOnce(&mut Self) -> Option<T>) -> Option<Option<T>> {
        let present = match self.u8()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let value = read(self)?;
        Some(present.then_some(value))
    }
}
