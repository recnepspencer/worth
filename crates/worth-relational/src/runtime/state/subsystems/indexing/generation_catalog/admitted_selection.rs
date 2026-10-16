use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexId, SelectedIndexGenerationAdmissionStop, SelectedIndexReadWork,
};

use super::{GenerationCatalog, GenerationScope};

impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn exact_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: crate::schema::data::SchemaVersionId,
        prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>>,
        SelectedIndexGenerationAdmissionStop<Stop>,
    > {
        let Some(scope) = self.scope_admitted(index, branch, prepare)? else {
            return Ok(None);
        };
        let Some(id) = scope.exact_admitted(version, schema, prepare)? else {
            return Ok(None);
        };
        let work = navigation_work(self.entries.len(), 1)?;
        prepare(SelectedIndexReadWork::OrderedNavigation(work), 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(self.generation(id))
    }

    pub(in crate::runtime::state::subsystems::indexing) fn published_for_commit_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        commit: CommitId,
        version: VersionId,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>>,
        SelectedIndexGenerationAdmissionStop<Stop>,
    > {
        let scope = self.scope_admitted(index, branch, &mut |work, bytes| {
            let (SelectedIndexReadWork::Operation(units)
            | SelectedIndexReadWork::OrderedNavigation(units)) = work;
            prepare(units, bytes)
        })?;
        let Some(scope) = scope else {
            return Ok(None);
        };
        let visits = navigation_work(scope.published_commit_count(), 2)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(id) = scope.published_for_commit(commit, version) else {
            return Ok(None);
        };
        let visits = navigation_work(self.entries.len(), 1)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(self.generation(id))
    }

    /// Selected metadata only, under the catalog's existing read lock.
    pub(in crate::runtime::state::subsystems::indexing) fn has_published_for_commit_admitted<
        Stop,
    >(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        commit: CommitId,
        version: VersionId,
        prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), Stop>,
    ) -> Result<bool, SelectedIndexGenerationAdmissionStop<Stop>> {
        let Some(scope) = self.scope_admitted(index, branch, prepare)? else {
            return Ok(false);
        };
        let entries = scope.published_commit_count();
        let payload = payload_comparison_bound(entries, 2)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let path = navigation_work(entries, 1)?;
        prepare(SelectedIndexReadWork::Operation(payload), 0)
            .and_then(|()| prepare(SelectedIndexReadWork::OrderedNavigation(path), 0))
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(scope.published_for_commit(commit, version).is_some())
    }

    fn scope_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), Stop>,
    ) -> Result<Option<&GenerationScope>, SelectedIndexGenerationAdmissionStop<Stop>> {
        // The outer scalar key excludes every unrelated index before any
        // branch payload is compared or copied.
        prepare(
            SelectedIndexReadWork::OrderedNavigation(navigation_work(self.scopes.len(), 1)?),
            0,
        )
        .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(branches) = self.scopes.get(&index) else {
            return Ok(None);
        };
        let branch_len = branch.map_or(0, |branch| branch.0.len());
        let key_visits = branch_len
            .checked_add(1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let payload = payload_comparison_bound(branches.len(), key_visits)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let copy_bytes = u64::try_from(branch_len)
            .map_err(|_| SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let path = navigation_work(branches.len(), 1)?;
        prepare(SelectedIndexReadWork::Operation(copy_bytes), copy_bytes)
            .and_then(|()| prepare(SelectedIndexReadWork::Operation(payload), 0))
            .and_then(|()| prepare(SelectedIndexReadWork::OrderedNavigation(path), 0))
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(branches.get(&branch.cloned()))
    }
}

/// One locked std B-tree descent visits at most eleven keys per level.
pub(in crate::runtime::state::subsystems::indexing) fn navigation_work<Stop>(
    entries: usize,
    key_visits: usize,
) -> Result<u64, SelectedIndexGenerationAdmissionStop<Stop>> {
    let levels = usize::BITS - entries.leading_zeros();
    u64::from(levels)
        .checked_mul(11)
        .and_then(|work| work.checked_mul(u64::try_from(key_visits).ok()?))
        .and_then(|work| work.checked_add(1))
        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)
}

// A single descent never compares a key twice. The held owner lock makes the
// selected container's current length a valid second bound on comparisons.
// Payload comparisons remain logical work; none is relabeled navigation.
fn payload_comparison_bound(entries: usize, key_visits: usize) -> Option<u64> {
    let levels = usize::BITS - entries.leading_zeros();
    let comparisons = u64::try_from(entries)
        .ok()?
        .min(u64::from(levels).checked_mul(11)?);
    comparisons.checked_mul(u64::try_from(key_visits.checked_sub(1)?).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn navigation_quote_preserves_the_typed_overflow_edge() {
        // Compute the edge independently with wider arithmetic, without a fake catalog.
        let descent = u128::from(usize::BITS) * 11;
        let safe = usize::try_from((u128::from(u64::MAX) - 1) / descent).unwrap();
        assert_eq!(
            navigation_work::<()>(usize::MAX, safe).unwrap(),
            u64::try_from(descent * safe as u128 + 1).unwrap()
        );
        assert!(matches!(
            navigation_work::<()>(usize::MAX, safe + 1),
            Err(SelectedIndexGenerationAdmissionStop::AccountingOverflow)
        ));
    }
}
