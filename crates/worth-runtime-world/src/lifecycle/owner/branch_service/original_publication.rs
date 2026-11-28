//! Source admission joins one exact Native effect to its original World occurrence.
use super::super::RuntimeWorldOwnerRoot;
use crate::history::{
    WorldObservedHomePublication, WorldReadmittedPublication,
    WorldReadmittedPublicationDenial as Denial,
};
use worth_foundational::facade::{
    DurableProductBranchReference, DurableProductReferenceGeneration,
};
use worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn read_verified_occurrence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: &ReadmittedRelationalAssociatedEffect,
    ) -> Result<
        (
            crate::durability::WorldCheckpointImage,
            crate::durability::AdmittedWorldOccurrence,
        ),
        Denial,
    > {
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
        let relational = occurrence.relational().ok_or(Denial::NativeMismatch)?;
        if matches.next().is_some()
            || relational.commit != effect.subject().receipt().commit_id.0
            || relational.position != effect.subject().position().0
            || &relational.digest != effect.subject().artifact_digest()
        {
            return Err(Denial::NativeMismatch);
        }
        let occurrence = occurrence.clone();
        Ok((image, occurrence))
    }

    pub(super) fn read_original_native_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: ReadmittedRelationalAssociatedEffect,
    ) -> Result<WorldReadmittedPublication, Denial> {
        let _call = self.try_admit_call().ok_or(Denial::OwnerUnavailable)?;
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::OwnerUnavailable)?;
        let (_, occurrence) = self.read_verified_occurrence(execution, &effect)?;
        self.state.history.readmitted_native_publication(
            execution,
            self.owner_identity(),
            &occurrence,
            effect,
        )
    }

    fn read_verified_occurrence_direct(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: &ReadmittedRelationalAssociatedEffect,
    ) -> Result<
        Option<(
            crate::durability::WorldOpenAttemptRecord,
            crate::durability::WorldOccurrenceOwnerEvidence,
            worth_foundational::facade::HomeIncarnation,
            u64,
            [u8; 32],
        )>,
        Denial,
    > {
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
        let Some((position, expected_digest)) =
            binding.occurrence_location(association.attempt().ordinal())
        else {
            return Ok(None);
        };
        let source = binding
            .capture_checkpoint_source(execution)
            .map_err(|cause| {
                Denial::Source(Box::new(crate::durability::WorldReadmissionDenial::Medium(
                    cause,
                )))
            })?;
        let record = source
            .read_journal_record(
                execution,
                worth_foundational::facade::WorldDurableJournal::Occurrences,
                position,
            )
            .map_err(|cause| {
                Denial::Source(Box::new(crate::durability::WorldReadmissionDenial::Medium(
                    cause,
                )))
            })?;
        if record.position() != position || record.digest() != expected_digest {
            return Err(Denial::NativeMismatch);
        }
        let decoded = crate::durability::decode_record(
            record.bytes(),
            binding.binding().owner().home(),
            execution,
        )
        .map_err(|cause| {
            Denial::Source(Box::new(crate::durability::WorldReadmissionDenial::Medium(
                cause,
            )))
        })?;
        let crate::durability::WorldRecord::Occurrence {
            open,
            owners,
            publishing_incarnation,
        } = &decoded.record
        else {
            return Err(Denial::NativeMismatch);
        };
        if open.association != association {
            return Err(Denial::NativeMismatch);
        }
        let relational = owners.relational().ok_or(Denial::NativeMismatch)?;
        if relational.commit != effect.subject().receipt().commit_id.0
            || relational.position != effect.subject().position().0
            || &relational.digest != effect.subject().artifact_digest()
        {
            return Err(Denial::NativeMismatch);
        }
        Ok(Some((
            open.clone(),
            owners.clone(),
            *publishing_incarnation,
            position,
            expected_digest,
        )))
    }

    pub(super) fn observe_original_home_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: ReadmittedRelationalAssociatedEffect,
        include_before_component: bool,
    ) -> Result<WorldObservedHomePublication, Denial> {
        let _call = self.try_admit_call().ok_or(Denial::OwnerUnavailable)?;
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::OwnerUnavailable)?;
        let expected_owner = self
            .owner_identity()
            .durable()
            .ok_or(Denial::NotReadmitted)?;
        let (
            open,
            owners,
            publishing_incarnation,
            journal_position,
            artifact_digest,
            recorded_before_component,
        ) = if include_before_component {
            let (image, occurrence) = self.read_verified_occurrence(execution, &effect)?;
            let open = occurrence.open().clone();
            let recorded_before_component =
                image.recorded_reference(&open.expected).and_then(|head| {
                    match &head.record().record {
                        crate::durability::WorldRecord::Branch { components, .. } => {
                            Some(components.relational.clone())
                        }
                        crate::durability::WorldRecord::BranchCreated(created) => {
                            Some(created.components.relational.clone())
                        }
                        _ => None,
                    }
                });
            (
                open,
                occurrence.owners().clone(),
                occurrence.publishing_incarnation(),
                occurrence.location().position(),
                *occurrence.location().digest(),
                recorded_before_component,
            )
        } else {
            let (open, owners, publishing_incarnation, position, digest) = self
                .read_verified_occurrence_direct(execution, &effect)?
                .ok_or(Denial::NotReadmitted)?;
            (open, owners, publishing_incarnation, position, digest, None)
        };
        if open.expected.branch.owner() != expected_owner || open.commit.owner() != expected_owner {
            return Err(Denial::ForeignHome);
        }
        let before = DurableProductBranchReference::from_recorded(
            open.expected.branch.clone(),
            open.expected.incarnation,
            open.expected.generation,
            open.expected.commit,
        )
        .map_err(|_| Denial::NativeMismatch)?;
        let next_generation = open
            .expected
            .generation
            .get()
            .checked_add(1)
            .ok_or(Denial::NativeMismatch)?;
        let after = DurableProductBranchReference::from_recorded(
            open.expected.branch.clone(),
            open.expected.incarnation,
            DurableProductReferenceGeneration::from_recorded(next_generation),
            open.commit,
        )
        .map_err(|_| Denial::NativeMismatch)?;
        let product_branch = crate::identity::ProductBranchIdentity::issued(
            self.owner_identity(),
            crate::branch::ProductBranchName::try_new(open.expected.branch.name().to_owned())
                .map_err(|_| Denial::NativeMismatch)?,
        );
        let product_incarnation = crate::identity::readmission::readmit_incarnation(
            self.owner_identity(),
            open.expected.incarnation,
        )
        .ok_or(Denial::ForeignHome)?;
        let product_generation =
            crate::identity::ProductBranchReferenceGeneration::readmitted(next_generation);
        let composite_commit =
            crate::identity::readmission::readmit_commit(self.owner_identity(), open.commit)
                .ok_or(Denial::ForeignHome)?;
        let publication_attempt = crate::identity::readmission::readmit_attempt(
            self.owner_identity(),
            open.association.attempt(),
        )
        .ok_or(Denial::ForeignHome)?;
        let signal_source = owners.signal().map(|actual| actual.point().clone());
        let signal_posture = if signal_source.is_some() {
            crate::history::CompositeComponentChangePosture::Published
        } else {
            crate::history::CompositeComponentChangePosture::RetainExact
        };
        Ok(WorldObservedHomePublication::from_verified_occurrence(
            before,
            after,
            product_branch,
            product_incarnation,
            product_generation,
            composite_commit,
            publication_attempt,
            signal_posture,
            signal_source,
            open.association.attempt(),
            journal_position,
            artifact_digest,
            publishing_incarnation,
            recorded_before_component,
            effect,
        ))
    }
}
