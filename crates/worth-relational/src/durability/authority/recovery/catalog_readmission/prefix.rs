//! Each root is acquired through its original canonical publication custody.
use super::*;
use crate::durability::authority::RelationalAssociatedEffectSubject;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    HomeProgramTransitionCaller, HomeProgramTransitionStepAssociation,
};

pub(super) fn admit(
    recovered: &RecoveredRelationalRuntimeAuthority,
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    original: &RelationalHomeProgramCatalogReadmissionSource<'_>,
    subjects: &[RelationalAssociatedEffectSubject],
    custody: &mut ReadmittedRelationalHomeProgramCatalogs,
) -> Result<(), Denial> {
    let mut previous = original.predecessor().clone();
    for (step, subject) in subjects.iter().enumerate() {
        request.consult().map_err(Denial::Request)?;
        let bytes = subject
            .association()
            .ok_or(Denial::PrefixMismatch)?
            .as_bytes();
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if digest != original.callers()[step] {
            return Err(Denial::PrefixMismatch);
        }
        let envelope = HomeProgramTransitionCaller::read(bytes).map_err(Denial::Caller)?;
        let coordinate = HomeProgramTransitionStepAssociation::from_recorded(
            original.plan().identity(),
            step as u32,
        );
        if envelope.step() != coordinate
            || envelope.catalog() != original.plan().target_meaning().catalog()
            || original
                .plan()
                .step_role(step as u32)
                .map_err(Denial::Plan)?
                != envelope.role()
            || original
                .plan()
                .step_commitment(coordinate, &original.assignments()[step])
                .map_err(Denial::Plan)?
                != envelope.commitment()
        {
            return Err(Denial::PrefixMismatch);
        }
        let effect = runtime
            .durability_authority()
            .readmit_associated_effect(request, subject)
            .map_err(Denial::AssociatedEffect)?;
        let successor = recovered
            .recorded_publication_successor(request, runtime, &effect)
            .map_err(Denial::Successor)?;
        let publication = recovered
            .admit_recorded_publication(request, runtime, effect, &previous, &successor)
            .map_err(Denial::Publication)?
            .consume(|publication| publication);
        custody.current = publication.successor().clone();
        *custody.prefix[step]
            .lock()
            .unwrap_or_else(|cause| cause.into_inner()) = Some(publication);
        previous = successor;
    }
    Ok(())
}
