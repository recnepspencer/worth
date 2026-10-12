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
    pin: std::sync::Arc<WorthQueryPreparedQueryPin>,
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
}
/// One immutable source pin per query side, shared with owner-held plans.
/// Private fields prevent re-pairing its root, layout, binding or basis.
pub(in crate::domain_computation) struct WorthQueryPreparedQueryPin {
    read: worth_relational::facade::runtime::RelationalPinnedRead,
    layout: std::sync::Arc<WorthQueryPrimaryGraphLayout>,
    basis: WorthQueryApplicationBasisIdentity,
    binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
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
        if read.snapshot().runtime_instance_id() != basis.runtime_instance_id()
            || read.snapshot().branch_id() != basis.branch_id()
            || read.snapshot().snapshot_id() != basis.snapshot_id()
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        Ok(WorthQueryPreparedSessionRead {
            pin: std::sync::Arc::new(WorthQueryPreparedQueryPin {
                read,
                layout,
                basis: basis.clone(),
                binding: self.binding.clone(),
            }),
            session: self.identity,
            plan: self.plan.identity(),
        })
    }
    /// Bind an existing side pin to this already admitted session's identity.
    /// This consults custody, never an owner port or current native registry.
    pub(in crate::domain_computation) fn prepare_query_read_from_pin(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        pin: &std::sync::Arc<WorthQueryPreparedQueryPin>,
    ) -> Result<WorthQueryPreparedSessionRead, WorthQueryManagedGraphReadDenial> {
        execution
            .check_live()
            .map_err(WorthQueryManagedGraphReadDenial::RequestStop)?;
        let WorthQueryGraphWorkBasis::Query { identity, .. } = &self.basis else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        if identity != basis
            || !self.branch.admits_query_basis(basis)
            // Operational snapshot ids differ between independently admitted
            // sessions on the same exact Native root and World occurrence.
            || pin.basis.runtime_instance_id() != basis.runtime_instance_id()
            || pin.basis.branch_id() != basis.branch_id()
            || pin.basis.descriptor() != basis.descriptor()
            || pin.basis.selection() != basis.selection()
            || snapshot.runtime_instance_id() != basis.runtime_instance_id()
            || snapshot.branch_id() != basis.branch_id()
            || snapshot.snapshot_id() != basis.snapshot_id()
            || snapshot.version_id() != pin.read.snapshot().version_id()
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        if pin.binding != self.binding {
            return Err(WorthQueryManagedGraphReadDenial::ForeignGraph);
        }
        Ok(WorthQueryPreparedSessionRead {
            pin: std::sync::Arc::clone(pin),
            session: self.identity,
            plan: self.plan.identity(),
        })
    }
}
impl WorthQueryPreparedSessionRead {
    pub(in crate::domain_computation) fn retained_pin(
        &self,
    ) -> std::sync::Arc<WorthQueryPreparedQueryPin> {
        std::sync::Arc::clone(&self.pin)
    }
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
        let output = read(&self.pin.read, &self.pin.layout);
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
        // The owner-held side pin owns the index map, layout and exact basis.
        // Its existing native-plan/index retention allowance funds that custody.
        // Workers share the whole pin through Arc; no map nodes or basis strings
        // are copied into this input, which adds only inline handles/identities.
        let Self {
            pin: _pin,
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
