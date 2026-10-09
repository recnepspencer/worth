//! Append the original performed-output footprint using the existing fact wire.

use super::super::facts;
use crate::domain_computation::primary_graph::{
    output_lineage::{invalidation::InvalidationEditAdmission, SealedNativeOutputWitness},
    WorthQueryApplicationObservedFact as Fact,
};

pub(super) fn encode(
    producer: &[Fact],
    witness: &SealedNativeOutputWitness,
    admission: &mut InvalidationEditAdmission,
) -> Option<Vec<u8>> {
    // Producer serialization retains checkpoint capture's existing bounded cold
    // encoding lane. This admission funds the new native projection and merge.
    // Unsupported producer facts retain the established nonreusable posture.
    let mut encoded = facts::encode(producer)?;
    let projection = witness.prepare_fact_projection(admission).ok()??;
    let count = producer.len().checked_add(projection.count())?;
    let fact_bytes = projection
        .count()
        .checked_mul(std::mem::size_of::<Fact>())?;
    let scratch = u64::try_from(fact_bytes)
        .ok()?
        .checked_add(projection.retained_payload_bytes())?;
    admission.admit_read_scratch(scratch).ok()?;
    admission
        .charge_external_work(u64::try_from(projection.count()).ok()?)
        .ok()?;
    let mut output = Vec::with_capacity(projection.count());
    projection.append_into(&mut output);
    let output_bytes = output.iter().try_fold(4usize, |bytes, fact| {
        admission.charge_external_work(1).ok()?;
        let width = match fact {
            Fact::Entity { .. } => 21,
            Fact::SourceAspectRevision {
                aspect,
                native_revision,
                ..
            } => 26usize
                .checked_add(aspect.as_str().len())?
                .checked_add(if native_revision.is_some() { 8 } else { 0 })?,
            _ => return None,
        };
        bytes.checked_add(width)
    })?;
    let total = encoded.len().checked_add(output_bytes.checked_sub(4)?)?;
    if total > facts::MAXIMUM_FACT_BYTES {
        return None;
    }
    // The bounded encoder and subsequent append both copy initialized wire
    // bytes; account for the output Vec and possible producer-Vec reallocation.
    let peak = encoded
        .capacity()
        .checked_add(total)?
        .checked_add(output_bytes)?;
    admission
        .admit_read_scratch(u64::try_from(peak).ok()?)
        .ok()?;
    admission
        .charge_external_work(u64::try_from(output_bytes.checked_add(total)?.checked_add(4)?).ok()?)
        .ok()?;
    let output_wire = facts::encode_with_capacity(&output, output_bytes)?;
    debug_assert_eq!(output_wire.len(), output_bytes);
    encoded
        .try_reserve_exact(total.checked_sub(encoded.len())?)
        .ok()?;
    encoded[..4].copy_from_slice(&u32::try_from(count).ok()?.to_be_bytes());
    encoded.extend_from_slice(&output_wire[4..]);
    Some(encoded)
}
