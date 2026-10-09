//! Read-only dispatch custody; the capacity-reserved session stays on its owner.

use super::super::{
    WorthQueryGraphReadOwnerPort, WorthQueryGraphWorkSessionIdentity,
    WorthQuerySessionGraphReadProof,
};
use super::{
    WorthQueryGraphWorkBasis, WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSession,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryPrimaryGraphLayout,
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// A move-only operation on an exact admitted session's source. This excludes
/// its Send-only capacity reservation and exposes no mutation capability.
pub(in crate::domain_computation) struct WorthQueryPreparedSessionRead<'a> {
    port: &'a WorthQueryGraphReadOwnerPort,
    binding: &'a ApplicationSchemaBindingIdentity,
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn prepare_query_read(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
    ) -> Result<WorthQueryPreparedSessionRead<'_>, WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::Query { identity, port, .. } = &self.basis else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        if identity != basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        Ok(WorthQueryPreparedSessionRead {
            port,
            binding: &self.binding,
            session: self.identity,
            plan: self.plan.identity(),
        })
    }
}

impl WorthQueryPreparedSessionRead<'_> {
    pub(in crate::domain_computation) fn execute<T>(
        self,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalRuntime,
            &WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQueryPreparedReadCompletion), WorthQueryManagedGraphReadDenial> {
        use super::super::owner_execution::WorthQueryGraphReadOwnerPortDenial as PortDenial;
        let output =
            self.port
                .execute_prepared(self.binding, read)
                .map_err(|cause| match cause {
                    PortDenial::ForeignGraph => WorthQueryManagedGraphReadDenial::ForeignGraph,
                })?;
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

impl worth_execution::ChargedBytes for WorthQueryPreparedSessionRead<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        // Shared source custody is retained by the owner-side session.
        let Self {
            port: _port,
            binding: _binding,
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
