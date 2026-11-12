//! Materialize creation strings only inside their original funded artifact.
use super::*;
use crate::branch::SignalBranchCreationPlan as Signal;
use crate::durability::artifact::WorldRecordedRelationalCreationPlan as Native;
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, history::BranchId, publication::PatchStreamPosition,
};

pub(crate) enum WorldRelationalCreationInput<'a> {
    ReuseExact,
    ForkExact {
        target: &'a BranchId,
        canonical_before: PatchStreamPosition,
        transition_before: RelationalBranchTransitionSequence,
    },
}

pub(crate) struct WorldCreationIntentInput<'a> {
    pub(crate) association: &'a CompositeAttemptAssociation,
    pub(crate) custody: DurableProductUnpublishedOwnerEffectsIdentity,
    pub(crate) source: WorldHeadInput<'a>,
    pub(crate) destination: WorldHeadInput<'a>,
    pub(crate) program_revision: [u8; 32],
    pub(crate) relational: WorldRelationalCreationInput<'a>,
    pub(crate) signal: &'a Signal,
}

impl<'a> WorldCreationIntentInput<'a> {
    pub(crate) fn from_record(record: &'a WorldCreationIntentRecord) -> Self {
        Self {
            association: &record.association,
            custody: record.custody,
            source: head(&record.source),
            destination: head(&record.destination),
            program_revision: record.program_revision,
            relational: match &record.relational {
                Native::ReuseExact => WorldRelationalCreationInput::ReuseExact,
                Native::ForkExact {
                    target,
                    canonical_before,
                    transition_before,
                } => WorldRelationalCreationInput::ForkExact {
                    target,
                    canonical_before: *canonical_before,
                    transition_before: *transition_before,
                },
            },
            signal: &record.signal,
        }
    }

    pub(super) fn name_bytes(&self) -> Result<usize, Denial> {
        let native = match self.relational {
            WorldRelationalCreationInput::ReuseExact => 0,
            WorldRelationalCreationInput::ForkExact { target, .. } => target.0.len(),
        };
        let signal = match self.signal {
            Signal::ReuseExact => 0,
            Signal::ForkExact { target } => target.as_str().len(),
        };
        [
            self.association.branch().name().len(),
            self.source.name.len(),
            self.destination.name.len(),
            native,
            signal,
        ]
        .into_iter()
        .try_fold(0usize, |total, bytes| total.checked_add(bytes))
        .ok_or_else(|| WorldDurableFrameDenial::SizeLimit.into())
    }

    pub(super) fn materialize(self) -> Result<WorldCreationIntentRecord, Denial> {
        let relational = match self.relational {
            WorldRelationalCreationInput::ReuseExact => Native::ReuseExact,
            WorldRelationalCreationInput::ForkExact {
                target,
                canonical_before,
                transition_before,
            } => Native::ForkExact {
                target: BranchId(super::copy_name(&target.0)?),
                canonical_before,
                transition_before,
            },
        };
        let signal = match self.signal {
            Signal::ReuseExact => Signal::ReuseExact,
            Signal::ForkExact { target } => Signal::ForkExact {
                target: worth_signal::facade::branch::validate_signal_branch_name(
                    super::copy_name(target.as_str())?,
                )
                .map_err(|_| WorldDurableFrameDenial::IdentityMismatch)?,
            },
        };
        let record = WorldCreationIntentRecord {
            association: copy_association(self.association)?,
            custody: self.custody,
            source: self.source.materialize()?,
            destination: self.destination.materialize()?,
            program_revision: self.program_revision,
            relational,
            signal,
        };
        record.validate_description()?;
        Ok(record)
    }
}

fn head(record: &DurableWorldHead) -> WorldHeadInput<'_> {
    WorldHeadInput {
        owner: record.branch.owner(),
        name: record.branch.name(),
        incarnation: record.incarnation,
        generation: record.generation,
        commit: record.commit,
    }
}

pub(super) fn copy_association(
    association: &CompositeAttemptAssociation,
) -> Result<CompositeAttemptAssociation, Denial> {
    CompositeAttemptAssociation::from_recorded(
        association.attempt(),
        super::branch(association.branch().owner(), association.branch().name())?,
        association.incarnation(),
    )
    .map_err(|_| WorldDurableFrameDenial::IdentityMismatch.into())
}
