//! Stable adoption family payload inside the version-two Query envelope.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::codec::Reader;
pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    bytes: &mut Vec<u8>,
    record: &QueryAdoptionRecord,
) {
    bytes.extend(
        rmp_serde::to_vec_named(record)
            .expect("adoption DATA contains only total primitive encodings"),
    );
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
) -> Result<QueryAdoptionRecord, QueryObligationDenial> {
    use serde::Deserialize;
    let mut cursor = std::io::Cursor::new(reader.0);
    let record = QueryAdoptionRecord::deserialize(&mut rmp_serde::Deserializer::new(&mut cursor))
        .map_err(|_| QueryObligationDenial::MalformedFrame)?;
    if cursor.position() != reader.0.len() as u64 {
        return Err(QueryObligationDenial::MalformedFrame);
    }
    reader.0 = &[];
    Ok(record)
}

// Only the codec maps the durable commit description to its stable DATA pair.
pub(super) mod commit {
    use serde::{Deserialize, Serialize};
    use worth_foundational::facade::{DurableCompositeCommitIdentity, DurableWorldOwnerIdentity};
    pub(in crate::domain_computation::primary_graph::query_obligations::adoption) fn serialize<
        S: serde::Serializer,
    >(
        value: &DurableCompositeCommitIdentity,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (value.owner(), value.ordinal()).serialize(serializer)
    }
    pub(in crate::domain_computation::primary_graph::query_obligations::adoption) fn deserialize<
        'de,
        D: serde::Deserializer<'de>,
    >(
        deserializer: D,
    ) -> Result<DurableCompositeCommitIdentity, D::Error> {
        let (owner, ordinal) = <(DurableWorldOwnerIdentity, u64)>::deserialize(deserializer)?;
        Ok(DurableCompositeCommitIdentity::from_recorded(
            owner, ordinal,
        ))
    }
}
