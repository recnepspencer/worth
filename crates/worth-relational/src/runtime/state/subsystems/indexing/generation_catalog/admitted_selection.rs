use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::SelectedIndexReadWork;
use crate::indexes::data::{DerivedIndexId, SelectedIndexGenerationAdmissionStop};

use super::GenerationCatalog;

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
        let branch_bytes = branch.map_or(0, |branch| branch.0.len());
        let key_visits = branch_bytes
            .checked_add(2)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let scope_work = payload_comparison_bound(key_visits)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let scope_path = navigation_work(self.scopes.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(
            SelectedIndexReadWork::Operation(branch_bytes as u64),
            branch_bytes as u64,
        )
        .and_then(|()| prepare(SelectedIndexReadWork::OrderedNavigation(scope_path), 0))
        .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        prepare(SelectedIndexReadWork::Operation(scope_work), 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(scope) = self.scope(index, branch) else {
            return Ok(None);
        };
        let id = scope.exact_admitted(version, schema, prepare)?;
        let Some(id) = id else {
            return Ok(None);
        };
        let work = navigation_work(self.entries.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
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
        let width = branch.map_or(0, |branch| branch.0.len());
        let scope_work = navigation_work(
            self.scopes.len(),
            width
                .checked_add(2)
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
        )
        .and_then(|work| work.checked_add(width as u64))
        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(scope_work, width as u64)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(scope) = self.scope(index, branch) else {
            return Ok(None);
        };
        let visits = navigation_work(scope.published_commit_count(), 2)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(id) = scope.published_for_commit(commit, version) else {
            return Ok(None);
        };
        let visits = navigation_work(self.entries.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(self.generation(id))
    }
    /// Look up only one already published generation. The admission remains
    /// under the same catalog read lock as the charged B-tree operations.
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
        let branch_len = branch.map_or(0, |branch| branch.0.len());
        let key_visits = branch_len
            .checked_add(2)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let scope_work = payload_comparison_bound(key_visits)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let copy_bytes = u64::try_from(branch_len)
            .map_err(|_| SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let scope_path = navigation_work(self.scopes.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(SelectedIndexReadWork::Operation(copy_bytes), copy_bytes)
            .and_then(|()| prepare(SelectedIndexReadWork::OrderedNavigation(scope_path), 0))
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        prepare(SelectedIndexReadWork::Operation(scope_work), 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(scope) = self.scope(index, branch) else {
            return Ok(false);
        };
        prepare(
            SelectedIndexReadWork::Operation(
                payload_comparison_bound(2)
                    .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
            ),
            0,
        )
        .and_then(|()| {
            prepare(
                SelectedIndexReadWork::OrderedNavigation(
                    navigation_work(scope.published_commit_count(), 1).unwrap(),
                ),
                0,
            )
        })
        .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(scope.published_for_commit(commit, version).is_some())
    }
}

/// A std B-tree node has at most eleven keys. A binary-tree height bound is
/// conservative for its minimum fanout and depends only on live owner count.
pub(in crate::runtime::state::subsystems::indexing) fn navigation_work(
    entries: usize,
    key_visits: usize,
) -> Option<u64> {
    let levels = usize::BITS.checked_sub(entries.leading_zeros())?;
    u64::from(levels)
        .checked_mul(11)?
        .checked_mul(u64::try_from(key_visits).ok()?)?
        .checked_add(1)
}

// The key's payload maximum is prepaid independently of unrelated catalog
// population; its actual single-descent path remains navigation. The largest
// addressable population bounds every declared catalog capacity.
fn payload_comparison_bound(key_visits: usize) -> Option<u64> {
    SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK
        .checked_mul(u64::try_from(key_visits.checked_sub(1)?).ok()?)
}
