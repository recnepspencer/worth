//! Atomic root obligation registration, including exact external observations.
use super::*;

impl RelationalBranchRetentionBinding {
    pub(crate) fn acquire(
        &self,
        kind: RelationalRetentionObligationKind,
        roots: Vec<Arc<RelationalBranchRoot>>,
        operation_branch: Option<RelationalBranchIdentity>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionAcquisitionDenial> {
        self.acquire_subject(kind, None, roots, operation_branch)
    }

    pub(crate) fn acquire_component(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionAcquisitionDenial> {
        self.acquire_subject(
            RelationalRetentionObligationKind::ExternalComponentBasis,
            Some(Arc::downgrade(&basis.inner)),
            vec![basis.inner.root.clone()],
            None,
        )
    }

    pub(crate) fn checkpoint_components(
        &self,
    ) -> Vec<crate::branch::AdmittedRelationalBranchBasis> {
        lock_lane(&self.lane)
            .obligations
            .values()
            .filter_map(|record| {
                record
                    .component
                    .as_ref()?
                    .upgrade()
                    .map(|inner| crate::branch::AdmittedRelationalBranchBasis { inner })
            })
            .collect()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    fn acquire_subject(
        &self,
        kind: RelationalRetentionObligationKind,
        component: Option<Weak<crate::branch::AdmittedRelationalBranchBasisInner>>,
        roots: Vec<Arc<RelationalBranchRoot>>,
        operation_branch: Option<RelationalBranchIdentity>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionAcquisitionDenial> {
        if roots.is_empty() || roots.len() > 2 {
            return Err(RelationalRetentionAcquisitionDenial::RootSetTooLarge);
        }
        let owner = self
            .owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        if let Some(operation_branch) = operation_branch.as_ref()
            && self.lane.identity.as_ref() != Some(operation_branch)
        {
            return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
        }
        reserve_live_obligation_capacity(&owner)?;
        let mut state = lock_lane(&self.lane);
        let obligation_id = match state.next_obligation_id.checked_add(1) {
            Some(id) => id,
            None => {
                drop(state);
                release_live_obligation_capacity(&owner);
                return Err(RelationalRetentionAcquisitionDenial::IdentityExhausted);
            }
        };
        state.next_obligation_id = obligation_id;
        let active_operation = operation_branch.is_some();
        if active_operation {
            state.active_operations = match state.active_operations.checked_add(1) {
                Some(count) => count,
                None => {
                    drop(state);
                    release_live_obligation_capacity(&owner);
                    return Err(RelationalRetentionAcquisitionDenial::IdentityExhausted);
                }
            };
        }
        let root_ids: Vec<_> = roots.iter().map(|root| root.id()).collect();
        state.retain_roots(&root_ids);
        state.obligations.insert(
            obligation_id,
            RelationalRetentionObligationRecord {
                kind,
                component,
                root_ids,
                active_operation,
            },
        );
        record_acquire(&mut state.counters, kind);
        owner.ordinary_counters.record_acquire(kind);
        Ok(RelationalRetentionGuard {
            owner: Arc::downgrade(&owner),
            lane: Arc::clone(&self.lane),
            owner_identity: Arc::as_ptr(&owner) as usize,
            obligation_id,
            roots,
            terminal: false,
        })
    }
}
