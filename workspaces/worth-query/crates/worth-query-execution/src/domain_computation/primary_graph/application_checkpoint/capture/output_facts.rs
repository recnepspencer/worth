//! Append the original performed-output footprint using the existing fact wire.
use super::super::facts::FactSequence;

use super::super::facts;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;

pub(super) fn encode(
    producer: &(impl facts::FactSequence + ?Sized),
    witness: &SealedNativeOutputWitness,
) -> Option<Vec<u8>> {
    // Producer serialization retains checkpoint capture's existing bounded cold
    // encoding lane. This admission funds the new native projection and merge.
    // Unsupported producer facts retain the established nonreusable posture.
    let mut encoded = facts::encode(producer)?;
    // This Query-local best-effort meter cannot reach a request stop.
    let projection = witness.prepare_fact_projection().ok()??;
    let count = producer.len().checked_add(projection.count())?;
    let mut output = Vec::with_capacity(projection.count());
    projection.append_into(&mut output);
    let output_bytes = output.iter().try_fold(4usize, |bytes, fact| {
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
    let output_wire = facts::encode_with_capacity(&output, output_bytes)?;
    debug_assert_eq!(output_wire.len(), output_bytes);
    encoded
        .try_reserve_exact(total.checked_sub(encoded.len())?)
        .ok()?;
    encoded[..4].copy_from_slice(&u32::try_from(count).ok()?.to_be_bytes());
    encoded.extend_from_slice(&output_wire[4..]);
    Some(encoded)
}
