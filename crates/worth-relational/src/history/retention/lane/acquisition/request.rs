//! Actual record/count construction precedes every lane accounting effect.
use super::*;
use worth_execution::{ExecutionAllocationPolicy, ExecutionOrderedMapDenial, ExecutionRequest};
#[derive(Debug)]
pub(crate) enum RelationalRetentionRequestAcquisitionDenial {
    Retention(RelationalRetentionAcquisitionDenial),
    Storage(ExecutionOrderedMapDenial),
    Request(crate::execution::RelationalExecutionDenialCause),
}
fn prepare<V>(
    key: u64,
    value: V,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<ExecutionOrderedMapCell<u64, V>, RelationalRetentionRequestAcquisitionDenial> {
    match request {
        Some(request) => {
            ExecutionOrderedMapCell::prepare_owned_native_for_request(key, value, *request)
                .map_err(RelationalRetentionRequestAcquisitionDenial::Storage)
        }
        None => Ok(ExecutionOrderedMapCell::prepare_owned_native_in(
            key,
            value,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("ordinary fixed native cell layout has no resource ceiling")),
    }
}
impl RelationalBranchRetentionBinding {
    pub(crate) fn acquire_observation_for_request(
        &self,
        root: Arc<RelationalBranchRoot>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionRequestAcquisitionDenial> {
        self.acquire_prepared_subject(
            RelationalRetentionObligationKind::Observation,
            None,
            RetainedRoots::one(root),
            None,
            Some(request),
        )
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "Address preserves the original retention owner identity and custody check"
    )]
    pub(super) fn acquire_prepared_subject(
        &self,
        kind: RelationalRetentionObligationKind,
        component: Option<Weak<crate::branch::AdmittedRelationalBranchBasisInner>>,
        roots: RetainedRoots,
        operation_branch: Option<RelationalBranchIdentity>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionRequestAcquisitionDenial> {
        use RelationalRetentionRequestAcquisitionDenial as Denial;
        if let Some(request) = request {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
        }
        let owner = self.owner.upgrade().ok_or(Denial::Retention(
            RelationalRetentionAcquisitionDenial::OwnerUnavailable,
        ))?;
        if operation_branch
            .as_ref()
            .is_some_and(|identity| self.lane.identity.as_ref() != Some(identity))
        {
            return Err(Denial::Retention(
                RelationalRetentionAcquisitionDenial::OwnerUnavailable,
            ));
        }
        if !super::super::super::owner::has_live_obligation_room(&owner) {
            return Err(Denial::Retention(
                RelationalRetentionAcquisitionDenial::CapacityExhausted,
            ));
        }
        let mut state = lock_lane(&self.lane);
        let id = state
            .next_obligation_id
            .checked_add(1)
            .ok_or(Denial::Retention(
                RelationalRetentionAcquisitionDenial::IdentityExhausted,
            ))?;
        let active = operation_branch.is_some();
        let operations = state
            .active_operations
            .checked_add(u64::from(active))
            .ok_or(Denial::Retention(
                RelationalRetentionAcquisitionDenial::IdentityExhausted,
            ))?;
        let ids = RootIds::from_roots(&roots);
        let mut prepared_counts = [None, None];
        let mut final_counts = [None, None];
        for (index, root) in ids.iter().enumerate() {
            if index == 1 && ids[0] == *root {
                continue;
            }
            let occurrences = ids.iter().filter(|other| *other == root).count();
            let prior = state.root_obligations.get(root).copied().unwrap_or(0);
            let count = prior.checked_add(occurrences).ok_or(Denial::Retention(
                RelationalRetentionAcquisitionDenial::CapacityExhausted,
            ))?;
            if prior == 0 {
                prepared_counts[index] = Some(prepare(*root, count, request)?);
            }
            final_counts[index] = Some((*root, count));
        }
        let record = prepare(
            id,
            RelationalRetentionObligationRecord::new(kind, component, ids, active),
            request,
        )?;
        if let Some(request) = request {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
        }
        reserve_live_obligation_capacity(&owner).map_err(Denial::Retention)?;
        // All storage/counters were preflighted. The maps are private and never
        // cloned; no fallible work follows publication of the actual record.
        state.next_obligation_id = id;
        state.active_operations = operations;
        for (index, final_count) in final_counts.into_iter().enumerate() {
            let Some((root, count)) = final_count else {
                continue;
            };
            if let Some(cell) = prepared_counts[index].take() {
                state
                    .root_obligations
                    .insert_owned_vacant_prepared(cell)
                    .expect("same unique count map still has the vacant root");
            } else {
                state
                    .root_obligations
                    .set_existing_unique(&root, count)
                    .expect("same unique count entry remains registered");
            }
        }
        state
            .obligations
            .insert_owned_vacant_prepared(record)
            .expect("same unique obligation map has the next unused id");
        record_acquire(&mut state.counters, kind);
        owner.ordinary_counters.record_acquire(kind);
        Ok(RelationalRetentionGuard {
            owner: Arc::downgrade(&owner),
            lane: Arc::clone(&self.lane),
            owner_identity: Arc::as_ptr(&owner) as usize,
            obligation_id: id,
            roots,
            terminal: false,
        })
    }
}
