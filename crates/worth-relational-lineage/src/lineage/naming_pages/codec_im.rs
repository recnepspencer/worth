//! `NamingCodec` implementations for persistent `im` collections, matching the
//! exact canonical wire format of `Vec`, `BTreeSet`, and `BTreeMap`.

use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::codec::{NamingCodec, Reader, Writer};

impl<T: NamingCodec + Clone> NamingCodec for im::Vector<T> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        self.iter().for_each(|value| value.write(out));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        (0..input.count()?).map(|_| T::read(input)).collect()
    }
}

impl<T: NamingCodec + Ord + Clone> NamingCodec for im::OrdSet<T> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        self.iter().for_each(|value| value.write(out));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let mut set = im::OrdSet::new();
        for _ in 0..input.count()? {
            let value = T::read(input)?;
            if set.get_max().is_some_and(|last| *last >= value) {
                return Err(Cause::Damaged);
            }
            set.insert(value);
        }
        Ok(set)
    }
}

impl<K: NamingCodec + Ord + Clone, V: NamingCodec + Clone> NamingCodec for im::OrdMap<K, V> {
    fn write(&self, out: &mut Writer) {
        out.u32(u32::try_from(self.len()).unwrap_or(u32::MAX));
        for (key, value) in self {
            key.write(out);
            value.write(out);
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let mut map = im::OrdMap::new();
        for _ in 0..input.count()? {
            let key = K::read(input)?;
            if map.get_max().is_some_and(|(last, _)| *last >= key) {
                return Err(Cause::Damaged);
            }
            map.insert(key, V::read(input)?);
        }
        Ok(map)
    }
}
