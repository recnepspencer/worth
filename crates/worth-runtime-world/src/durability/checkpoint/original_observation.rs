//! Descriptive original identity joins use the existing protected World image.
use super::WorldProgramActivationHistory;
use crate::durability::{AdmittedWorldOccurrence, WorldReadmissionDenial, WorldRecord};
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositeCommitIdentity, DurableProductBranchIdentity,
};

/// Refusals keep current-Home admission distinct from missing source and request stops.
#[derive(Debug)]
pub enum WorldOriginalObservationDenial {
    Incarnation(worth_proof::HomeIncarnationDenial),
    Source(WorldReadmissionDenial),
    Request(worth_execution::WorkCeilingDenial),
}

/// Borrowed original metadata, independent of optional live material retention.
/// This lends neither an executable receipt nor disclosure or mutation authority.
#[derive(Debug)]
pub struct WorldOriginalObservation<'a> {
    original: &'a AdmittedWorldOccurrence,
}
impl WorldOriginalObservation<'_> {
    pub fn identity(&self) -> DurableCompositeCommitIdentity {
        self.original.open().commit
    }
    pub fn predecessor(&self) -> DurableCompositeCommitIdentity {
        self.original.open().expected.commit
    }
    pub fn branch(&self) -> &DurableProductBranchIdentity {
        &self.original.open().expected.branch
    }
    pub fn relational_commit(&self) -> worth_relational::facade::history::CommitId {
        worth_relational::facade::history::CommitId(self.original.relational().commit)
    }
    pub fn program_revision(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(self.original.open().program_revision)
    }
}
impl WorldProgramActivationHistory {
    /// Borrow the already admitted originals of this one protected image and tails.
    /// No history lease, source replacement, publication or user code is entered.
    pub fn original_observations(&self) -> impl Iterator<Item = WorldOriginalObservation<'_>> {
        self.image
            .occurrences
            .values()
            .map(|original| WorldOriginalObservation { original })
    }

    /// Check original ancestry in the authoritative image, including disposed live
    /// occurrences. The selected identity must belong to this current Home owner.
    /// O(selected ancestry) read-only work; it never admits the original for use.
    pub fn original_is_ancestor_of(
        &self,
        original: &WorldOriginalObservation<'_>,
        selected: &crate::identity::CompositeCommitIdentity,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, WorldOriginalObservationDenial> {
        use worth_proof::HomeIncarnationDenial;
        use WorldOriginalObservationDenial as Denial;
        let owner = selected.owner_identity();
        let expected = &self.image.binding;
        if owner.durable() != Some(expected.owner())
            || original.identity().owner() != expected.owner()
        {
            return Err(Denial::Incarnation(HomeIncarnationDenial::ForeignHome));
        }
        let fence = owner
            .home_incarnation_fence()
            .ok_or(Denial::Incarnation(HomeIncarnationDenial::ForeignHome))?;
        expected
            .incarnation()
            .fence()
            .admit(fence)
            .map_err(Denial::Incarnation)?;
        let candidate = original.identity();
        let mut position =
            DurableCompositeCommitIdentity::from_recorded(expected.owner(), selected.ordinal());
        loop {
            request.consult().map_err(Denial::Request)?;
            let head = self
                .image
                .history
                .get(&position)
                .ok_or(Denial::Source(WorldReadmissionDenial::BranchLineage))?;
            if position == candidate {
                // A view captured before publication cannot establish an original
                // from a later cut, even if its descriptive Home identity agrees.
                return Ok(self
                    .image
                    .occurrences
                    .get(&original.original.location().position())
                    .is_some_and(|entry| {
                        entry.open() == original.original.open()
                            && entry.relational() == original.original.relational()
                    }));
            }
            match &head.program_record().record {
                WorldRecord::Branch { .. } => return Ok(false),
                WorldRecord::OpenAttempt(open) => position = open.expected.commit,
                WorldRecord::Owner { .. }
                | WorldRecord::Occurrence { .. }
                | WorldRecord::Released { .. }
                | WorldRecord::Continued { .. } => {
                    unreachable!("validated history stores only origins and original opens")
                }
            }
        }
    }
}
