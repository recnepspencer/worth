//! Price selected occurrence walks before a registry mutation starts.
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::{infallible, occurrence_rows, OccurrenceSearchCharge, WalkAdmission, WalkKind};
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use core::convert::Infallible;

/// Exclusive custody spans payment and execution. These edits may remove
/// rows, but must neither insert rows nor grow an occurrence before walking.
/// The existing immutable whole-map receipt is a different proof and keeps
/// its immutable borrow. This receipt pays occurrence-local upper bounds.
pub(in super::super) struct PricedMutation<'a> {
    state: &'a mut DemandRegistryState,
    charge: PricedMutationWalk,
}

struct PricedMutationWalk {
    paths: u64,
    rows: u64,
}

pub(in super::super) struct SourceWalkAllowance {
    pub paths: u64,
    pub rows: u64,
    pub entries: u64,
    pub navigation: u64,
}

pub(in super::super) fn price_mutation<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    walks: &[(WorthQueryOutputDemandKey, u64)],
    kind: WalkKind,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    price_mutation_with_source_allowance(
        state,
        admission,
        walks,
        kind,
        SourceWalkAllowance {
            paths: 0,
            rows: 0,
            entries: 0,
            navigation: 0,
        },
    )
}

pub(in super::super) fn price_mutation_with_source_allowance<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    walks: &[(WorthQueryOutputDemandKey, u64)],
    kind: WalkKind,
    allowance: SourceWalkAllowance,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    price_walks(
        state,
        admission,
        walks.iter().map(|(key, count)| (key, *count)),
        kind,
        allowance,
    )
}

pub(in super::super) fn price_member_release<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    key: &WorthQueryOutputDemandKey,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    price_walks(
        state,
        admission,
        std::iter::once((key, 1)),
        WalkKind::RequiredMember,
        SourceWalkAllowance {
            paths: 0,
            rows: 0,
            entries: 0,
            navigation: 0,
        },
    )
}

fn price_walks<'a, 'keys>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    walks: impl IntoIterator<Item = (&'keys WorthQueryOutputDemandKey, u64)>,
    kind: WalkKind,
    allowance: SourceWalkAllowance,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    let mut paths = allowance.paths;
    let mut rows = allowance.rows;
    let mut navigation = allowance.navigation;
    for (key, count) in walks {
        let mut occurrence_count = 0u64;
        // The quote itself is a paid walk, including its row inspections.
        let mut charge = WalkAdmission::new(admission, kind);
        let navigation_before = charge.admission_mut().charged_navigation();
        let selected = occurrence_rows(&state.records, key, &mut charge)?;
        let path_navigation = charge.admission_mut().charged_navigation() - navigation_before;
        for _ in selected {
            charge.inspect_row()?;
            occurrence_count = occurrence_count
                .checked_add(1)
                .ok_or_else(|| kind.denial())?;
        }
        let inspected = occurrence_count
            .checked_mul(count)
            .ok_or_else(|| kind.denial())?;
        navigation = path_navigation
            .checked_mul(count)
            .and_then(|price| navigation.checked_add(price))
            .ok_or_else(|| kind.denial())?;
        paths = paths.checked_add(count).ok_or_else(|| kind.denial())?;
        rows = rows.checked_add(inspected).ok_or_else(|| kind.denial())?;
    }
    let rows = rows
        .checked_add(allowance.entries)
        .ok_or_else(|| kind.denial())?;
    let work = paths
        .checked_mul(2)
        .and_then(|price| price.checked_add(rows))
        .ok_or_else(|| kind.denial())?;
    admission
        .charge_ordered_operations(work, navigation)
        .map_err(|_| kind.denial())?;
    Ok(PricedMutation {
        state,
        charge: PricedMutationWalk { paths, rows },
    })
}

impl PricedMutation<'_> {
    pub(in super::super) fn prune_custody_entry(
        &mut self,
        commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        infallible(self.state.prune_custody_entry(commit, &mut self.charge));
    }

    pub(in super::super) fn retire_closed_rows(
        &mut self,
        candidates: Vec<WorthQueryOutputDemandKey>,
    ) -> (
        Vec<super::super::DemandRecord>,
        Vec<std::sync::Arc<WorthQueryOutputDemandKey>>,
    ) {
        infallible(self.state.retire_closed_rows(candidates, &mut self.charge))
    }

    pub(in super::super) fn prune_closed_note(
        &mut self,
        queued: super::super::cleanup::closed_notes::QueuedClosedRow,
        candidates: Vec<WorthQueryOutputDemandKey>,
    ) -> (
        Vec<super::super::DemandRecord>,
        Vec<std::sync::Arc<WorthQueryOutputDemandKey>>,
    ) {
        let note = self.state.take_closed_note(queued);
        note.prune(|state| {
            for key in &candidates {
                if let Some(mut row) = state.records.get_mut(key) {
                    if row.terminal()
                        && row.interests == 0
                        && row.framework_required_count == 0
                        && row.prepared_prerequisite_claims == 0
                    {
                        let released = row.release_obligations();
                        state.obligation_reserved_bytes =
                            state.obligation_reserved_bytes.saturating_sub(released);
                    }
                }
            }
            // The cascade can end the last stale owner of another row in this
            // occurrence. Its member is debt of this same paid visit too.
            let retired =
                infallible(state.retire_closed_rows(candidates.clone(), &mut self.charge));
            for key in &candidates {
                infallible(state.remove_required_member_if_released(key, &mut self.charge));
            }
            retired
        })
    }

    pub(in super::super) fn complete<T>(
        mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, Infallible>,
    ) -> T {
        infallible(operation(&mut self))
    }

    pub(in super::super) fn state(&self) -> &DemandRegistryState {
        self.state
    }

    pub(in super::super) fn state_mut(&mut self) -> &mut DemandRegistryState {
        self.state
    }

    pub(in super::super) fn release_prerequisites(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Vec<std::sync::Arc<WorthQueryOutputDemandKey>> {
        infallible(
            self.state
                .release_record_prerequisites(key, &mut self.charge),
        )
    }

    pub(in super::super) fn release_prerequisites_detached(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        members: &mut Vec<super::super::required_members::DetachedRequiredMember>,
    ) -> super::super::obligations::ReleasedPrerequisiteResources {
        infallible(
            self.state
                .release_record_prerequisites_detached(key, members, &mut self.charge),
        )
    }

    pub(in super::super) fn release_member(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> super::super::required_members::DetachedRequiredMember {
        infallible(
            self.state
                .remove_required_member_if_released_detached(key, &mut self.charge),
        )
    }

    pub(in super::super) fn follow_newest(
        &mut self,
        dependent: &WorthQueryOutputDemandKey,
        upstream: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandKey> {
        infallible(
            self.state
                .follow_newest(dependent, upstream, &mut self.charge),
        )
    }

    pub(in super::super) fn retire_row(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        retired: &mut Vec<super::super::DemandRecord>,
    ) -> Vec<std::sync::Arc<WorthQueryOutputDemandKey>> {
        infallible(self.state.retire_row(key, retired, &mut self.charge))
    }
}

impl OccurrenceSearchCharge for PricedMutationWalk {
    type Refusal = Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Infallible> {
        self.paths = self
            .paths
            .checked_sub(1)
            .expect("selected mutation walks were priced");
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Infallible> {
        self.rows = self
            .rows
            .checked_sub(1)
            .expect("selected occurrence rows were priced");
        Ok(())
    }
}
