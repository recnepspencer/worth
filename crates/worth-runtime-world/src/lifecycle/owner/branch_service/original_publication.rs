//! Source admission joins one exact Native effect to its original World occurrence.
use super::super::RuntimeWorldOwnerRoot;
use crate::history::{WorldReadmittedPublication, WorldReadmittedPublicationDenial as Denial};
use worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn read_original_native_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: ReadmittedRelationalAssociatedEffect,
    ) -> Result<WorldReadmittedPublication, Denial> {
        let _call = self.try_admit_call().ok_or(Denial::OwnerUnavailable)?;
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::OwnerUnavailable)?;
        let binding = self.state.durable.as_ref().ok_or(Denial::NotReadmitted)?;
        let association = effect
            .subject()
            .association()
            .ok_or(Denial::NativeMismatch)?;
        let association = crate::durability::decode_association(
            association.as_bytes(),
            binding.binding().owner().home(),
        )
        .map_err(|_| Denial::NativeMismatch)?;
        let source = binding
            .capture_checkpoint_source(execution)
            .map_err(|cause| {
                Denial::Source(Box::new(crate::durability::WorldReadmissionDenial::Medium(
                    cause,
                )))
            })?;
        let (image, _) = crate::durability::WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            binding.binding(),
            execution,
            None,
        )
        .map_err(|cause| Denial::Source(Box::new(cause)))?;
        let mut matches = image
            .occurrences
            .iter()
            .filter(|(_, row)| row.open().association == association);
        let (_, occurrence) = matches.next().ok_or(Denial::NotReadmitted)?;
        if matches.next().is_some()
            || occurrence.relational().commit != effect.subject().receipt().commit_id.0
            || occurrence.relational().position != effect.subject().position().0
            || &occurrence.relational().digest != effect.subject().artifact_digest()
        {
            return Err(Denial::NativeMismatch);
        }
        self.state.history.readmitted_native_publication(
            execution,
            self.owner_identity(),
            occurrence,
            effect,
        )
    }
}
