//! Canonical byte codec of naming-page keys and values.
//!
//! A page decodes only after its content address matched a member its
//! checkpoint admitted, exactly as lineage event pages do; rows then decode
//! strictly (no trailing bytes, ascending sets and maps).
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::{
    PersistentNameHolder, PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

/// Appends canonical bytes.
#[derive(Default)]
pub struct Writer(crate::naming_wire::Writer);

impl Writer {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
    pub fn u8(&mut self, value: u8) {
        self.0.u8(value);
    }
    pub fn u32(&mut self, value: u32) {
        self.0.u32(value);
    }
    pub fn u64(&mut self, value: u64) {
        self.0.u64(value);
    }
    pub fn digest(&mut self, digest: &[u8; 32]) {
        self.0.digest(digest);
    }
    pub fn bytes(&mut self, bytes: &[u8]) {
        self.u32(u32::try_from(bytes.len()).unwrap_or(u32::MAX));
        self.0.raw(bytes);
    }
}

/// Reads canonical bytes; every shortfall or overrun is damage.
pub struct Reader<'a>(crate::naming_wire::Reader<'a>);

impl<'a> Reader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self(crate::naming_wire::Reader::new(bytes))
    }

    pub fn u8(&mut self) -> Result<u8, Cause> {
        self.0.u8().map_err(|_| Cause::Damaged)
    }

    pub fn u32(&mut self) -> Result<u32, Cause> {
        self.0.u32().map_err(|_| Cause::Damaged)
    }

    pub fn u64(&mut self) -> Result<u64, Cause> {
        self.0.u64().map_err(|_| Cause::Damaged)
    }

    pub fn digest(&mut self) -> Result<[u8; 32], Cause> {
        self.0.digest().map_err(|_| Cause::Damaged)
    }

    pub fn bytes(&mut self) -> Result<&'a [u8], Cause> {
        let count = usize::try_from(self.u32()?).map_err(|_| Cause::Damaged)?;
        self.0.take(count).map_err(|_| Cause::Damaged)
    }
    /// A count no larger than the bytes left, so a forged count cannot
    /// reserve memory the page never carried.
    pub fn count(&mut self) -> Result<usize, Cause> {
        let count = usize::try_from(self.u32()?).map_err(|_| Cause::Damaged)?;
        if count > self.0.remaining() {
            return Err(Cause::Damaged);
        }
        Ok(count)
    }
    pub fn finish(self) -> Result<(), Cause> {
        if self.0.remaining() == 0 {
            Ok(())
        } else {
            Err(Cause::Damaged)
        }
    }
}

/// One key or value of a naming table.
pub trait NamingCodec: Sized {
    fn write(&self, out: &mut Writer);
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause>;

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Writer::default();
        self.write(&mut out);
        out.into_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Cause> {
        let mut input = Reader::new(bytes);
        let value = Self::read(&mut input)?;
        input.finish()?;
        Ok(value)
    }
}

/// A serde record (record ids, member carriers) as one nested value.
pub fn write_serde(value: &impl serde::Serialize, out: &mut Writer) {
    out.bytes(&rmp_serde::encode::to_vec(value).unwrap_or_default());
}

pub fn read_serde<T: serde::de::DeserializeOwned>(input: &mut Reader<'_>) -> Result<T, Cause> {
    rmp_serde::from_slice(input.bytes()?).map_err(|_| Cause::Damaged)
}

impl NamingCodec for u64 {
    fn write(&self, out: &mut Writer) {
        out.u64(*self);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        input.u64()
    }
}

impl NamingCodec for LineageId {
    fn write(&self, out: &mut Writer) {
        out.u64(self.0);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self(input.u64()?))
    }
}

impl NamingCodec for CommitId {
    fn write(&self, out: &mut Writer) {
        out.u64(self.0);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self(input.u64()?))
    }
}

impl NamingCodec for () {
    fn write(&self, _: &mut Writer) {}
    fn read(_: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(())
    }
}

impl NamingCodec for String {
    fn write(&self, out: &mut Writer) {
        out.bytes(self.as_bytes());
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        String::from_utf8(input.bytes()?.to_vec()).map_err(|_| Cause::Damaged)
    }
}

impl NamingCodec for crate::contracts::history::data::coordinates::BranchId {
    fn write(&self, out: &mut Writer) {
        self.0.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self(String::read(input)?))
    }
}

impl NamingCodec for [u8; 32] {
    fn write(&self, out: &mut Writer) {
        out.digest(self);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        input.digest()
    }
}

impl NamingCodec for PersistentNameIdentity {
    fn write(&self, out: &mut Writer) {
        out.digest(self.as_bytes());
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self::from_minted_digest(input.digest()?))
    }
}

impl NamingCodec for AggregateIdentity {
    fn write(&self, out: &mut Writer) {
        out.digest(self.bytes());
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self::from_bytes(input.digest()?))
    }
}

impl NamingCodec for LineageSubject {
    fn write(&self, out: &mut Writer) {
        out.u8(match self {
            Self::Entity => 0,
            Self::Relation => 1,
        });
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Entity),
            1 => Ok(Self::Relation),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for PersistentNameKey {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Entity(name) => (out.u8(0), name.write(out)).1,
            Self::Relation(name) => (out.u8(1), name.write(out)).1,
            Self::Group(group) => (out.u8(2), group.write(out)).1,
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Entity(NamingCodec::read(input)?)),
            1 => Ok(Self::Relation(NamingCodec::read(input)?)),
            2 => Ok(Self::Group(NamingCodec::read(input)?)),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for PersistentNameHolder {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Entity(entity) => (out.u8(0), write_serde(entity, out)).1,
            Self::Relation(relation) => (out.u8(1), write_serde(relation, out)).1,
            Self::Lineage(lineage) => (out.u8(2), out.u64(lineage.0)).1,
            Self::PendingBirth {
                occurrence_key,
                role_path,
                ordinal,
            } => {
                out.u8(3);
                out.bytes(occurrence_key.as_bytes());
                out.bytes(role_path.as_bytes());
                out.u64(*ordinal as u64);
            }
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Entity(read_serde(input)?)),
            1 => Ok(Self::Relation(read_serde(input)?)),
            2 => Ok(Self::Lineage(LineageId(input.u64()?))),
            3 => Ok(Self::PendingBirth {
                occurrence_key: String::read(input)?,
                role_path: String::read(input)?,
                ordinal: usize::try_from(input.u64()?).map_err(|_| Cause::Damaged)?,
            }),
            _ => Err(Cause::Damaged),
        }
    }
}

impl<A: NamingCodec, B: NamingCodec> NamingCodec for (A, B) {
    fn write(&self, out: &mut Writer) {
        self.0.write(out);
        self.1.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok((A::read(input)?, B::read(input)?))
    }
}

impl<A: NamingCodec, B: NamingCodec, C: NamingCodec> NamingCodec for (A, B, C) {
    fn write(&self, out: &mut Writer) {
        self.0.write(out);
        self.1.write(out);
        self.2.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok((A::read(input)?, B::read(input)?, C::read(input)?))
    }
}

impl<T: NamingCodec> NamingCodec for Option<T> {
    fn write(&self, out: &mut Writer) {
        match self {
            None => out.u8(0),
            Some(value) => (out.u8(1), value.write(out)).1,
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(None),
            1 => Ok(Some(T::read(input)?)),
            _ => Err(Cause::Damaged),
        }
    }
}

impl<T: NamingCodec> NamingCodec for Vec<T> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        self.iter().for_each(|value| value.write(out));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        (0..input.count()?).map(|_| T::read(input)).collect()
    }
}

/// Sets and maps encode ascending and decode strictly ascending, so one
/// logical value has exactly one encoding.
impl<T: NamingCodec + Ord> NamingCodec for BTreeSet<T> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        self.iter().for_each(|value| value.write(out));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let mut set = BTreeSet::new();
        for _ in 0..input.count()? {
            let value = T::read(input)?;
            if set.last().is_some_and(|last| *last >= value) {
                return Err(Cause::Damaged);
            }
            set.insert(value);
        }
        Ok(set)
    }
}

impl<K: NamingCodec + Ord, V: NamingCodec> NamingCodec for BTreeMap<K, V> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        for (key, value) in self {
            key.write(out);
            value.write(out);
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let mut map = BTreeMap::new();
        for _ in 0..input.count()? {
            let key = K::read(input)?;
            if map.last_key_value().is_some_and(|(last, _)| *last >= key) {
                return Err(Cause::Damaged);
            }
            map.insert(key, V::read(input)?);
        }
        Ok(map)
    }
}
