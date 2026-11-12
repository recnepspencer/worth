//! The image retains a bijection onto each complete original journal prefix.
use super::*;
use crate::durability::WorldReadmissionDenial as Denial;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

impl WorldCheckpointImage {
    pub(super) fn validate_image_coordinates(
        &self,
        coverage: &WorldCheckpointCoverage,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let mut branches = ExecutionOrderedMap::default();
        if let Some(owner) = self.owner {
            insert(&mut branches, owner.position, owner.digest, request)?;
        }
        for branch in self.branches.keys() {
            insert(&mut branches, branch.position, branch.digest, request)?;
        }
        validate(&branches, &coverage.branches, request)?;
        let mut custody = ExecutionOrderedMap::default();
        for open in self.opens.values() {
            insert(&mut custody, open.position(), *open.digest(), request)?;
        }
        for disposition in self.dispositions.values() {
            insert(
                &mut custody,
                disposition.position(),
                *disposition.digest(),
                request,
            )?;
        }
        for intent in self.creations.values() {
            insert(&mut custody, intent.position(), *intent.digest(), request)?;
        }
        for release in self.creation_releases.values() {
            insert(&mut custody, release.position(), *release.digest(), request)?;
        }
        validate(&custody, &coverage.custody, request)?;
        if self.occurrences.len() as u64 != coverage.occurrences.position {
            return Err(Denial::JournalSourceMismatch);
        }
        if let Some((position, last)) = self
            .occurrences
            .entry_at(self.occurrences.len().saturating_sub(1))
        {
            if *position != coverage.occurrences.position
                || last.location().digest() != &coverage.occurrences.digest
            {
                return Err(Denial::JournalSourceMismatch);
            }
        } else if coverage.occurrences != WorldJournalCoverage::default() {
            return Err(Denial::JournalSourceMismatch);
        }
        Ok(())
    }
}
fn insert(
    map: &mut ExecutionOrderedMap<u64, [u8; 32]>,
    position: u64,
    digest: [u8; 32],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if position == 0 || map.contains_key(&position) {
        return Err(Denial::JournalSourceMismatch);
    }
    map.insert_cloned_native_for_request(&position, &digest, *request)
        .map_err(|cause| {
            Denial::Medium(crate::durability::WorldDurableMediumDenial::Ordered(cause))
        })
}
fn validate(
    map: &ExecutionOrderedMap<u64, [u8; 32]>,
    coverage: &WorldJournalCoverage,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if map.len() as u64 != coverage.position {
        return Err(Denial::JournalSourceMismatch);
    }
    let mut next = 0u64;
    let mut last_digest = [0; 32];
    for (position, digest) in map.iter() {
        super::framing::charge(request, 1).map_err(Denial::Medium)?;
        next = next.checked_add(1).ok_or(Denial::JournalSourceMismatch)?;
        if *position != next {
            return Err(Denial::JournalSourceMismatch);
        }
        last_digest = *digest;
    }
    if last_digest != coverage.digest {
        return Err(Denial::JournalSourceMismatch);
    }
    Ok(())
}
