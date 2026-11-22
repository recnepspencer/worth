//! Foundational native carriers whose serde form is a fixed compound.
//!
//! The temporal carriers are recognized by their exact serialized struct and
//! field names, `[u8; 16]` UUIDs by their 16-element tuple, and `InlineBytes`
//! by its transparent byte sequence. Any other compound remains unsupported,
//! and a carrier whose payload is not canonical refuses before effects.
use super::super::{ApplicationCanonicalInputBytes, ApplicationCanonicalInputScalar};
use super::{tag, Cursor, Denial};
use worth_foundational::facade::{
    AspectValue, CanonicalDate, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz,
    InlineBytes, ScalarAspectType,
};

/// Compound members are framed by these markers in the canonical grammar.
const END: u8 = 0;
const MEMBER: u8 = 1;
const UUID_BYTES: usize = 16;

impl<'a> Cursor<'a> {
    /// Selects a carrier-shaped family; `None` leaves other families unread.
    pub(super) fn native_carrier(
        &mut self,
        family: ScalarAspectType,
    ) -> Result<Option<ApplicationCanonicalInputScalar<'a>>, Denial> {
        use ScalarAspectType as F;
        let expected = match family {
            F::Date | F::Time | F::Timestamp | F::TimestampTz => tag::STRUCT,
            F::Uuid => tag::TUPLE,
            F::InlineBytes => tag::SEQ,
            _ => return Ok(None),
        };
        if self.byte()? != expected {
            return Err(Denial::InvalidScalar);
        }
        let selected = match family {
            F::Uuid => {
                ApplicationCanonicalInputScalar::Inline(AspectValue::Uuid(self.uuid_carrier()?))
            }
            F::InlineBytes => ApplicationCanonicalInputScalar::Bytes(self.byte_sequence_carrier()?),
            _ => {
                let value = self.temporal_carrier()?;
                if value.value_family() != family {
                    return Err(Denial::InvalidScalar);
                }
                ApplicationCanonicalInputScalar::Inline(value)
            }
        };
        Ok(Some(selected))
    }

    /// Reads and admits a temporal carrier after its STRUCT tag.
    pub(super) fn temporal_carrier(&mut self) -> Result<AspectValue, Denial> {
        let value = match self.temporal_members()? {
            TemporalMembers::Date(days) => AspectValue::Date(CanonicalDate {
                days_from_unix_epoch: narrow(days)?,
            }),
            TemporalMembers::Time(nanos) => {
                AspectValue::Time(CanonicalTime::new(narrow(nanos)?).ok_or(Denial::InvalidScalar)?)
            }
            TemporalMembers::Timestamp(micros) => AspectValue::Timestamp(CanonicalTimestamp {
                micros_since_unix_epoch: narrow(micros)?,
            }),
            TemporalMembers::TimestampTz(micros, offset) => {
                AspectValue::TimestampTz(CanonicalTimestampTz {
                    utc_micros_since_unix_epoch: narrow(micros)?,
                    offset_minutes: narrow(offset)?,
                })
            }
        };
        if !value.has_canonical_representation() {
            return Err(Denial::InvalidScalar);
        }
        Ok(value)
    }

    /// Reads a temporal carrier's grammar without admitting its value, as an
    /// unselected member is skipped; only a selected member is admitted.
    pub(super) fn temporal_members(&mut self) -> Result<TemporalMembers, Denial> {
        let members = match self.text()? {
            "CanonicalDate" => TemporalMembers::Date(self.signed_member("days_from_unix_epoch")?),
            "CanonicalTime" => TemporalMembers::Time(self.unsigned_member("nanos_since_midnight")?),
            "CanonicalTimestamp" => {
                TemporalMembers::Timestamp(self.signed_member("micros_since_unix_epoch")?)
            }
            "CanonicalTimestampTz" => TemporalMembers::TimestampTz(
                self.signed_member("utc_micros_since_unix_epoch")?,
                self.signed_member("offset_minutes")?,
            ),
            _ => return Err(Denial::UnsupportedShape),
        };
        if self.byte()? != END {
            return Err(Denial::UnsupportedShape);
        }
        Ok(members)
    }

    /// Reads a `[u8; 16]` after its TUPLE tag.
    pub(super) fn uuid_carrier(&mut self) -> Result<[u8; UUID_BYTES], Denial> {
        let mut uuid = [0_u8; UUID_BYTES];
        for byte in &mut uuid {
            *byte = self.sequence_byte()?.ok_or(Denial::InvalidScalar)?;
        }
        if self.byte()? != END {
            return Err(Denial::InvalidScalar);
        }
        Ok(uuid)
    }

    /// Borrows a byte sequence after its SEQ tag. O(encoded bytes); the copy
    /// waits for materialization under the caller's reservation.
    pub(super) fn byte_sequence_carrier(
        &mut self,
    ) -> Result<ApplicationCanonicalInputBytes<'a>, Denial> {
        let start = self.0;
        let mut length = 0_usize;
        while self.sequence_byte()?.is_some() {
            length += 1;
        }
        InlineBytes::admit_length(length).map_err(|_| Denial::InvalidScalar)?;
        let encoded = &start[..start.len() - self.0.len()];
        Ok(ApplicationCanonicalInputBytes::new(encoded, length))
    }

    /// One `u8` element of a sequence or tuple, or `None` at its end.
    fn sequence_byte(&mut self) -> Result<Option<u8>, Denial> {
        match self.byte()? {
            END => Ok(None),
            MEMBER if self.byte()? == tag::UNSIGNED => u8::try_from(self.integer()?)
                .map(Some)
                .map_err(|_| Denial::InvalidScalar),
            _ => Err(Denial::UnsupportedShape),
        }
    }

    fn signed_member(&mut self, name: &str) -> Result<i128, Denial> {
        self.member(name, tag::SIGNED)?;
        let bits = self.integer()?;
        Ok(((bits >> 1) as i128) ^ -((bits & 1) as i128))
    }

    fn unsigned_member(&mut self, name: &str) -> Result<u128, Denial> {
        self.member(name, tag::UNSIGNED)?;
        self.integer()
    }

    fn member(&mut self, name: &str, scalar_tag: u8) -> Result<(), Denial> {
        if self.byte()? != MEMBER || self.text()? != name {
            return Err(Denial::UnsupportedShape);
        }
        if self.byte()? != scalar_tag {
            return Err(Denial::InvalidScalar);
        }
        Ok(())
    }
}

/// A temporal carrier's members as serialized, before narrowing.
pub(super) enum TemporalMembers {
    Date(i128),
    Time(u128),
    Timestamp(i128),
    TimestampTz(i128, i128),
}

/// Copies a byte sequence that selection already validated.
pub(in super::super) fn materialize_byte_sequence(encoded: &[u8], length: usize) -> Vec<u8> {
    let mut cursor = Cursor(encoded);
    let mut bytes = Vec::with_capacity(length);
    while let Some(byte) = cursor
        .sequence_byte()
        .expect("selection validated the byte sequence")
    {
        bytes.push(byte);
    }
    bytes
}

fn narrow<Wide, Narrow: TryFrom<Wide>>(value: Wide) -> Result<Narrow, Denial> {
    Narrow::try_from(value).map_err(|_| Denial::InvalidScalar)
}
