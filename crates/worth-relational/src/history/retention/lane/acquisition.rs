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
        reason = "Addresses identify original live lane allocations for local snapshot dedup only"
    )]
    pub(in crate::history::retention) fn checkpoint_lane_identity(&self) -> usize {
        Arc::as_ptr(&self.lane) as usize
    }
    pub(in crate::history::retention) fn capture_checkpoint_components_into(
        &self,
        lane: usize,
        held: &mut ExecutionOrderedMap<(usize, u64), crate::branch::AdmittedRelationalBranchBasis>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRequestAcquisitionDenial> {
        use RelationalRetentionRequestAcquisitionDenial as Denial;
        let state = lock_lane(&self.lane);
        for (&id, record) in state.obligations.iter() {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            if let Some(inner) = record.component.as_ref().and_then(Weak::upgrade) {
                let basis = crate::branch::AdmittedRelationalBranchBasis { inner };
                if !Weak::ptr_eq(&basis.inner.retention_binding.owner, &self.owner) {
                    return Err(Denial::Retention(
                        RelationalRetentionAcquisitionDenial::BindingMismatch,
                    ));
                }
                held.insert_cloned_native_for_request(&(lane, id), &basis, *request)
                    .map_err(Denial::Storage)?;
            }
        }
        Ok(())
    }
    fn acquire_subject(
        &self,
        kind: RelationalRetentionObligationKind,
        component: Option<Weak<crate::branch::AdmittedRelationalBranchBasisInner>>,
        roots: Vec<Arc<RelationalBranchRoot>>,
        operation_branch: Option<RelationalBranchIdentity>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionAcquisitionDenial> {
        let roots = RetainedRoots::from_vec(roots)
            .ok_or(RelationalRetentionAcquisitionDenial::RootSetTooLarge)?;
        self.acquire_prepared_subject(kind, component, roots, operation_branch, None)
            .map_err(|cause| match cause {
                RelationalRetentionRequestAcquisitionDenial::Retention(cause) => cause,
                _ => unreachable!("ordinary fixed cells have no execution request refusal"),
            })
    }
}
mod request;
pub(crate) use request::RelationalRetentionRequestAcquisitionDenial;
