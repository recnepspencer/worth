use serde::{Deserialize, Serialize};

use super::PersistentVector;

impl<T: Clone + Serialize, const PAGE_LEN: usize> Serialize for PersistentVector<T, PAGE_LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'de, T: Clone + Deserialize<'de>, const PAGE_LEN: usize> Deserialize<'de>
    for PersistentVector<T, PAGE_LEN>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<T>::deserialize(deserializer).map(|values| values.into_iter().collect())
    }
}
