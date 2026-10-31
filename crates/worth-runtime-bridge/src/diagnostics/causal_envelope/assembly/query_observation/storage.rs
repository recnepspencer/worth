//! Fixed one-reference container backing, including temporary Arc conversion coexistence.
use super::super::super::{BridgeCausalEvidenceBinding, BridgeCausalEvidenceReference};
use super::identity::BridgeQueryObservationReportingStop as Failure;
use std::{alloc::Layout, sync::atomic::AtomicUsize};

pub(super) fn admit_containers<Stop>(
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<(), Failure<Stop>> {
    let reference = size_of::<BridgeCausalEvidenceReference>();
    let reference_arc = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<BridgeCausalEvidenceReference>())
        .map_err(|_| Failure::AccountingOverflow)?
        .0
        .pad_to_align()
        .size();
    let bytes = reference
        .checked_add(reference_arc)
        .and_then(|bytes| bytes.checked_add(size_of::<BridgeCausalEvidenceBinding>()))
        .ok_or(Failure::AccountingOverflow)?;
    admit(1, bytes).map_err(Failure::Resource)
}
