//! Bounded, required-only replay of an installed native discontinuity.

mod admission;

use admission::{
    admission_denial, capacity_denial, charge_outer_retirement, charge_split, charge_tree, levels,
    node_bytes, work_denial,
};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;

use worth_relational::facade::history::{BranchId, CommitId};
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::super::required_custody::RequiredOutputCustodyCapacity;
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct DiscontinuityCursors
{
    occurrences: BTreeMap<ProductBranchIncarnation, BranchCursors>,
    // std's empty ordered root can outlive its last member.
    _root_capacity: Option<RequiredOutputCustodyCapacity>,
}

pub(super) enum DiscontinuityPage {
    Queued,
    Advanced,
    Complete,
}

struct BranchCursors {
    branches: BTreeMap<BranchId, DiscontinuityCursor>,
    _root_capacity: Option<RequiredOutputCustodyCapacity>,
    _capacity: RequiredOutputCustodyCapacity,
    _retirement_paid: PrepaidCursorRetirement,
}

/// Removed under the registry lock and destroyed only after that lock exits.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct RetiredDiscontinuityCursors
{
    _row: BranchCursors,
}

struct DiscontinuityCursor {
    epoch: CommitId,
    last_key: Option<(
        Arc<WorthQueryOutputDemandKey>,
        RequiredOutputCustodyCapacity,
    )>,
    complete: bool,
    _capacity: RequiredOutputCustodyCapacity,
    _teardown_paid: PrepaidCursorRetirement,
}

/// Minted only after the creation request pays for this row's later no-fail
/// Product retirement. The installed required-byte cap bounds future height.
struct PrepaidCursorRetirement;

impl DemandRegistryState {
    /// A source discontinuity permits an exceptional walk of actual required
    /// keys. The cursor is indexed by both World occurrence and native cell.
    /// An unrelated or historical branch epoch cannot acknowledge this one.
    pub(super) fn page_required_discontinuity(
        &mut self,
        occurrence: ProductBranchIncarnation,
        branch: &BranchId,
        epoch: CommitId,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<DiscontinuityPage, WorthQueryOutputDemandDenial> {
        let outer_len = self.discontinuity_cursors.occurrences.len();
        charge_tree(admission, outer_len, 3, 2)?;
        let row = self.discontinuity_cursors.occurrences.get(&occurrence);
        let inner_len = row.map_or(0, |row| row.branches.len());
        charge_tree(
            admission,
            inner_len,
            branch.0.len().checked_add(2).ok_or_else(work_denial)?,
            2,
        )?;
        let previous = row.and_then(|row| row.branches.get(branch));
        let continuing = previous.is_some_and(|cursor| cursor.epoch == epoch);
        if previous.is_some_and(|cursor| continuing && cursor.complete) {
            return Ok(DiscontinuityPage::Complete);
        }
        let last = previous
            .filter(|_| continuing)
            .and_then(|cursor| cursor.last_key.as_ref())
            .map(|(key, _)| key);
        let required_len = self.required_keys.len();
        let selected = if let Some(last) = last {
            let comparison = last.producer.len().checked_add(7).ok_or_else(work_denial)?;
            charge_tree(admission, required_len, comparison, 1)?;
            self.required_keys
                .range((Excluded(Arc::clone(last)), Unbounded))
                .next()
        } else {
            admission
                .charge_external_work(
                    u64::try_from(levels(required_len)?).map_err(|_| work_denial())?,
                )
                .map_err(|_| work_denial())?;
            self.required_keys.iter().next()
        };
        admission
            .charge_external_work(2)
            .map_err(|_| work_denial())?;
        let selected_key = selected.cloned();
        let selected_member = if let Some(key) = selected_key.as_ref() {
            self.charge_record_lookup(key, admission)?;
            self.records.get(key.as_ref()).and_then(|record| {
                (record.product_occurrence == occurrence)
                    .then(|| record.work_membership.as_ref().cloned())
                    .flatten()
            })
        } else {
            None
        };

        // All new map nodes, branch text, and the independently pinned key
        // are admitted before the first cursor/index mutation or allocation.
        let outer_new = row.is_none();
        let inner_new = previous.is_none();
        let outer_bytes = outer_new
            .then(|| node_bytes::<ProductBranchIncarnation, BranchCursors>())
            .transpose()?;
        let root_bytes = (outer_new && self.discontinuity_cursors._root_capacity.is_none())
            .then(|| node_bytes::<ProductBranchIncarnation, BranchCursors>())
            .transpose()?;
        let inner_bytes = inner_new
            .then(|| {
                node_bytes::<BranchId, DiscontinuityCursor>().and_then(|bytes| {
                    bytes
                        .checked_add(branch.0.len())
                        .ok_or_else(capacity_denial)
                })
            })
            .transpose()?;
        let inner_root_bytes = (inner_new && row.is_none_or(|row| row._root_capacity.is_none()))
            .then(|| node_bytes::<BranchId, DiscontinuityCursor>())
            .transpose()?;
        let key_bytes = selected_key
            .as_ref()
            .map(|key| pinned_key_bytes(key.as_ref()))
            .transpose()?;
        // The Arc pin shares the already allocated key; it needs lifetime
        // custody, while only new branch text and tree nodes need scratch.
        let outer_peak = outer_bytes
            .map(|bytes| {
                bytes
                    .checked_mul(
                        levels(outer_len)?
                            .checked_add(1)
                            .ok_or_else(capacity_denial)?,
                    )
                    .ok_or_else(capacity_denial)
            })
            .transpose()?;
        let inner_peak = inner_bytes
            .map(|bytes| {
                bytes
                    .checked_mul(
                        levels(inner_len)?
                            .checked_add(1)
                            .ok_or_else(capacity_denial)?,
                    )
                    .ok_or_else(capacity_denial)
            })
            .transpose()?;
        let new_bytes = [outer_peak, root_bytes, inner_peak, inner_root_bytes]
            .into_iter()
            .flatten()
            .try_fold(0usize, |total, bytes| {
                total.checked_add(bytes).ok_or_else(capacity_denial)
            })?;
        admission
            .admit_read_scratch(u64::try_from(new_bytes).map_err(|_| capacity_denial())?)
            .map_err(admission_denial)?;
        let outer_ticket = outer_bytes
            .map(|bytes| self.reserve_required_custody_capacity(bytes))
            .transpose()?;
        let root_ticket = root_bytes
            .map(|bytes| self.reserve_required_custody_capacity(bytes))
            .transpose()?;
        let inner_ticket = inner_bytes
            .map(|bytes| self.reserve_required_custody_capacity(bytes))
            .transpose()?;
        let inner_root_ticket = inner_root_bytes
            .map(|bytes| self.reserve_required_custody_capacity(bytes))
            .transpose()?;
        let key_ticket = key_bytes
            .map(|bytes| self.reserve_required_custody_capacity(bytes))
            .transpose()?;
        if inner_new {
            admission
                .charge_external_work(
                    u64::try_from(branch.0.len().checked_add(2).ok_or_else(work_denial)?)
                        .map_err(|_| work_denial())?,
                )
                .map_err(|_| work_denial())?;
        }
        // Insertion is another real search at each new ordered level.
        if outer_new {
            charge_tree(admission, outer_len, 3, 1)?;
            charge_split(admission, outer_len)?;
            charge_outer_retirement(admission, self.required_budget_bytes)?;
        }
        if inner_new {
            charge_tree(
                admission,
                inner_len,
                branch.0.len().checked_add(2).ok_or_else(work_denial)?,
                1,
            )?;
            charge_split(admission, inner_len)?;
            // Dropping the branch-name key, cursor and one ordered entry is
            // charged to the creation that adds that exact future row.
            admission
                .charge_external_work(5)
                .map_err(|_| work_denial())?;
        }
        if selected_member.is_some() {
            // Membership version, active test, queue link and retained cause.
            admission
                .charge_external_work(5)
                .map_err(|_| work_denial())?;
        }
        let cursors = &mut self.discontinuity_cursors;
        if let Some(ticket) = root_ticket {
            cursors._root_capacity = Some(ticket);
        }
        if outer_new {
            cursors.occurrences.insert(
                occurrence,
                BranchCursors {
                    branches: BTreeMap::new(),
                    _root_capacity: None,
                    _capacity: outer_ticket.expect("new outer row has admitted custody"),
                    _retirement_paid: PrepaidCursorRetirement,
                },
            );
        }
        let row = cursors
            .occurrences
            .get_mut(&occurrence)
            .expect("selected cursor row exists");
        if let Some(ticket) = inner_root_ticket {
            row._root_capacity = Some(ticket);
        }
        if inner_new {
            row.branches.insert(
                branch.clone(),
                DiscontinuityCursor {
                    epoch,
                    last_key: None,
                    complete: false,
                    _capacity: inner_ticket.expect("new branch cursor has admitted custody"),
                    _teardown_paid: PrepaidCursorRetirement,
                },
            );
        }
        let cursor = row
            .branches
            .get_mut(branch)
            .expect("selected branch cursor exists");
        if cursor.epoch != epoch {
            cursor.epoch = epoch;
            cursor.last_key = None;
            cursor.complete = false;
        }
        cursor.last_key =
            selected_key.map(|key| (key, key_ticket.expect("selected key has admitted custody")));
        cursor.complete = cursor.last_key.is_none();
        if let Some(member) = selected_member {
            member.mark_discontinuity();
            return Ok(DiscontinuityPage::Queued);
        }
        Ok(if cursor.complete {
            DiscontinuityPage::Complete
        } else {
            DiscontinuityPage::Advanced
        })
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn take_discontinuity_cursors(
        &mut self,
        occurrence: ProductBranchIncarnation,
    ) -> Option<RetiredDiscontinuityCursors> {
        // This is the Query Product-retirement cleanup lane. An occurrence
        // without a cursor can be presented again by pending_cleanup; its
        // negative search costs O(log(maximum cursor rows)), where that finite
        // maximum follows from installed required-ledger bytes divided by the
        // mandatory minimum outer-row claim. It is cleanup-lane work, not a
        // request charge or cursor-creation prepayment. Present-row removal and each
        // inner-row teardown use the row's retained prepaid custody.
        self.discontinuity_cursors
            .occurrences
            .remove(&occurrence)
            .map(|row| RetiredDiscontinuityCursors { _row: row })
    }
}

fn pinned_key_bytes(
    key: &WorthQueryOutputDemandKey,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    std::mem::size_of::<WorthQueryOutputDemandKey>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(key.producer.len()))
        .ok_or_else(capacity_denial)
}
