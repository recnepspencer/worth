//! Changed input is refused before admission and carried with its retained state.
use super::{
    scope, DecomposeFailure, DecomposeInputDenial, DecomposeKernelEditions, DecomposeRunFailure,
    ExecutionDecompose, ExecutionMap, PartitionIdentity,
};

impl<I, C, S, B, O, F> ExecutionDecompose<I, C, S, B, O, F> {
    pub(super) fn check_changed<'a, T, K, E>(
        &'a self,
        changed: &'a ExecutionMap<T, K>,
        editions: DecomposeKernelEditions,
    ) -> Result<CheckedChangedInput<'a, Self, T, K>, DecomposeRunFailure<E>> {
        self.check_changed_identities(changed.identities(), editions)?;
        Ok(CheckedChangedInput {
            owner: self,
            changed,
            editions,
        })
    }

    pub(super) fn check_changed_identities<E>(
        &self,
        identities: &[PartitionIdentity],
        editions: DecomposeKernelEditions,
    ) -> Result<(), DecomposeRunFailure<E>> {
        self.validate_changed(identities, editions)
            .map_err(|cause| DecomposeRunFailure {
                cause: DecomposeFailure::Input(cause),
                total_report: scope::empty_report(),
            })
    }

    fn validate_changed(
        &self,
        identities: &[PartitionIdentity],
        editions: DecomposeKernelEditions,
    ) -> Result<(), DecomposeInputDenial> {
        if identities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(DecomposeInputDenial::ChangedIdentitiesNotCanonical);
        }
        if self.snapshot.is_none() && identities != self.identities.as_slice() {
            return Err(DecomposeInputDenial::InitialCoverageMismatch);
        }
        if self
            .snapshot
            .as_ref()
            .is_some_and(|previous| previous.editions.interior != editions.interior)
            && identities != self.identities.as_slice()
        {
            return Err(DecomposeInputDenial::InteriorKernelCoverageMismatch);
        }
        for identity in identities {
            if self.identities.as_slice().binary_search(identity).is_err() {
                return Err(DecomposeInputDenial::UnknownIdentity(*identity));
            }
        }
        Ok(())
    }
}

/// Only `check_changed` binds a map and editions to their borrowed retained state.
pub(super) struct CheckedChangedInput<'a, Owner, T, K> {
    owner: &'a Owner,
    changed: &'a ExecutionMap<T, K>,
    editions: DecomposeKernelEditions,
}

impl<'a, Owner, T, K> CheckedChangedInput<'a, Owner, T, K> {
    pub(super) fn map(&self) -> &'a ExecutionMap<T, K> {
        self.changed
    }

    pub(super) fn into_parts(self) -> (&'a Owner, &'a ExecutionMap<T, K>, DecomposeKernelEditions) {
        (self.owner, self.changed, self.editions)
    }
}

#[cfg(test)]
mod tests;
