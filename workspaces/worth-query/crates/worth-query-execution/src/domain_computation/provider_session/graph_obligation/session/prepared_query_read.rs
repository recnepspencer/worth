//! Read-only dispatch custody; the capacity-reserved session stays on its owner.

use super::super::{WorthQueryGraphWorkSessionIdentity, WorthQuerySessionGraphReadProof};
use super::{
    WorthQueryGraphWorkBasis, WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSession,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryPrimaryGraphLayout,
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;

/// A move-only operation on an exact admitted session's source. This excludes
/// its Send-only capacity reservation and exposes no mutation capability.
pub(in crate::domain_computation) struct WorthQueryPreparedSessionRead {
    read: worth_relational::facade::runtime::RelationalPinnedRead,
    layout: std::sync::Arc<WorthQueryPrimaryGraphLayout>,
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn prepare_query_read(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        select_indexes: impl FnOnce(
            &WorthQueryPrimaryGraphLayout,
        ) -> Vec<worth_relational::facade::indexes::DerivedIndexId>,
    ) -> Result<WorthQueryPreparedSessionRead, WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::Query { identity, port, .. } = &self.basis else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        if identity != basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let (read, layout) =
            port.pin_prepared(execution, &self.binding, snapshot, select_indexes)?;
        Ok(WorthQueryPreparedSessionRead {
            read,
            layout,
            session: self.identity,
            plan: self.plan.identity(),
        })
    }
}

impl WorthQueryPreparedSessionRead {
    pub(in crate::domain_computation) fn execute<T>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalPinnedRead,
            &WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQueryPreparedReadCompletion), WorthQueryManagedGraphReadDenial> {
        execution
            .check_live()
            .map_err(WorthQueryManagedGraphReadDenial::RequestStop)?;
        let output = read(&self.read, &self.layout);
        Ok((
            output,
            WorthQueryPreparedReadCompletion {
                session: self.session,
                plan: self.plan,
            },
        ))
    }
}

/// Move-only evidence that the sealed capability executed its read. The owner
/// supplies the retained basis when it seals this evidence, after the join.
pub(in crate::domain_computation) struct WorthQueryPreparedReadCompletion {
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
}
impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn seal_prepared_read(
        &self,
        completion: WorthQueryPreparedReadCompletion,
    ) -> Result<WorthQuerySessionGraphReadProof, WorthQueryManagedGraphReadDenial> {
        if completion.session != self.identity || completion.plan != self.plan.identity() {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let WorthQueryGraphWorkBasis::Query { identity, .. } = &self.basis else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        Ok(WorthQuerySessionGraphReadProof::new(
            completion.session,
            completion.plan,
            identity.clone(),
        ))
    }
}

impl worth_execution::ChargedBytes for WorthQueryPreparedSessionRead {
    fn additional_charged_bytes(&self) -> u64 {
        // Shared source custody is retained by the owner-side session.
        let Self {
            read: _read,
            layout: _layout,
            session: _session,
            plan: _plan,
        } = self;
        0
    }
}
impl worth_execution::ChargedBytes for WorthQueryPreparedReadCompletion {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            session: _session,
            plan: _plan,
        } = self;
        0
    }
}
