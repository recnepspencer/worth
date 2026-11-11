//! Prefix-free Full history serializes earlier rows directly from original DATA.
use crate::storage::residency::region_capture::complete::arena::{MetadataSlice, WireKind};
use crate::storage::substrate::RecordArena;
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};
pub(in crate::storage::residency) struct Rows<'a, K: WireKind>(
    pub(in crate::storage::residency) &'a RecordArena<K>,
);
impl<K: WireKind> Serialize for Rows<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Row<'a, K: WireKind>(MetadataSlice<'a, K>);
        impl<K: WireKind> Serialize for Row<'_, K> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut fields = serializer.serialize_struct("RegionHistoryRow", 2)?;
                fields.serialize_field("prefix", &Option::<super::RegionHistoryBase>::None)?;
                fields.serialize_field("earlier", &self.0)?;
                fields.end()
            }
        }
        let mut sequence = serializer.serialize_seq(Some(self.0.metadata_history.len()))?;
        for physical in 0..self.0.metadata_history.len() {
            let rows = &self.0.metadata_history[physical];
            sequence.serialize_element(&Row::<K>(MetadataSlice {
                rows,
                range: 0..rows.len().saturating_sub(1),
            }))?;
        }
        sequence.end()
    }
}
