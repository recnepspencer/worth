//! Distinct provider-purpose custody around a genuine admitted Parent Query.
use super::*;
use crate::{
    basis::{
        WorthQueryProductBranchAdmissionDenial as ProductDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    },
    domain_computation::{
        execution_runtime::product_world::WorthQueryProductRuntime,
        provider_session::WorthQueryCompletedSourceRead,
    },
};
use std::sync::Arc;
use worth_query_admission::facade::{
    authenticated_principal::{WorthQueryRequestInterruption, WorthQueryRequestScope},
    graph_obligation::WorthQueryGraphWorkIntentKind,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Debug)]
pub(in crate::domain_computation) enum WorthQueryManagedIndexSessionDenial {
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(WorthQueryRequestInterruption),
    Execution(Cause),
    Parent(WorthQueryManagedGraphReadDenial),
    AuthorityMismatch,
    Product(ProductDenial),
    TerminalReleaseMismatch,
}

/// No Native read port: the borrowed Parent retains its own Product loan.
pub(in crate::domain_computation) struct WorthQueryManagedIndexReadSession<'a, 'request, 'authority>
{
    execution: worth_execution::ExecutionRequest<'request, 'authority>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    product: &'a WorthQueryProductRuntime,
    request: &'a WorthQueryRequestScope,
    association: WorthQueryInstalledManagedIndexParent,
    source: Arc<WorthQueryCompletedSourceRead>,
    identity: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryAdmittedGraphWorkPlan,
}

pub(in crate::domain_computation) struct WorthQueryManagedIndexCallbackProof {
    session: WorthQueryGraphWorkSessionIdentity,
    plan: worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
}

/// Callback completion only; the root owner must still bind epoch and slots.
pub(in crate::domain_computation) struct WorthQueryManagedIndexSelectionProof<
    'a,
    'request,
    'authority,
> {
    execution: worth_execution::ExecutionRequest<'request, 'authority>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: WorthQueryInstalledManagedIndexParent,
    source: Arc<WorthQueryCompletedSourceRead>,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
    release: worth_query_admission::integration::WorthQueryExecutionCapacityReleaseReceipt,
}

impl<'a, 'request, 'authority> WorthQueryManagedIndexReadSession<'a, 'request, 'authority> {
    pub(in crate::domain_computation) fn identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.identity
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start(
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        product: &'a WorthQueryProductRuntime,
        request: &'a WorthQueryRequestScope,
        association: WorthQueryInstalledManagedIndexParent,
        source: Arc<WorthQueryCompletedSourceRead>,
        reserved: WorthQueryPreparedProductReadSessionIdentity,
        plan: WorthQueryAdmittedGraphWorkPlan,
    ) -> Result<Self, WorthQueryManagedIndexSessionDenial> {
        use WorthQueryManagedIndexSessionDenial as Denial;
        let mut returned = None;
        let outer = execution.run_native_with_checkpoint(|context| {
            returned = Some((|| {
                let same_source = source.admits_parent(
                    parent,
                    &mut |work| {
                        if context.has_accounting() {
                            context.checkpoint(work)
                        } else {
                            context.poll()
                        }
                        .map_err(|stop| {
                            Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop))
                        })
                    },
                    || Denial::Execution(Cause::WorkCounterOverflow),
                )?;
                let access = association.access_contract();
                if source.runtime() != parent.runtime
                    || source.principal() != parent.principal
                    || Some(source.scope()) != parent.entity_access_context()
                    || source.binding() != association.binding_identity()
                    || parent.binding() != association.binding_identity()
                    || source.obligation() != access.source_obligation()
                    || parent.obligation() != access.parent_obligation()
                    || !same_source
                    || plan.intent().kind() != WorthQueryGraphWorkIntentKind::ManagedIndexRead
                    || plan.binding_identity() != association.binding_identity()
                    || plan.obligation_identity() != access.graph_obligations().identity()
                {
                    return Err(Denial::AuthorityMismatch);
                }
                let (identity, managed_run) = reserved.into_identities();
                let session = Self {
                    execution: *execution,
                    parent,
                    product,
                    request,
                    association,
                    source,
                    identity,
                    managed_run,
                    plan,
                };
                session.check_current()?;
                Ok(session)
            })());
        });
        match (outer, returned) {
            (Ok(_), Some(result)) => result,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(Denial::Request(cause)),
            (Ok(_), None) => unreachable!("completed original Index session admission"),
        }
    }

    fn check_current(&self) -> Result<(), WorthQueryManagedIndexSessionDenial> {
        use WorthQueryManagedIndexSessionDenial as Denial;
        self.execution.consult().map_err(Denial::Request)?;
        if let Some(stop) = self.request.interruption() {
            return Err(Denial::Interrupted(stop));
        }
        let read = self.parent.query_read_basis().map_err(Denial::Parent)?;
        let crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(selected) = read.identity.selection() else {
            return Err(Denial::AuthorityMismatch);
        };
        self.product
            .with_product_observation(selected.branch_identity(), |actual| {
                if !selected.matches_observation(&actual) {
                    return Err(ProductDenial::Refused(
                        WorthQueryProductBranchAdmissionRefusalKind::ObservationStaleSourceHead,
                    ));
                }
                Ok(())
            })
            .map_err(Denial::Product)
    }

    /// Providers run after currentness, outside both Native and registry borrows.
    pub(in crate::domain_computation) fn call<T, E>(
        &self,
        callback: impl FnOnce(&worth_execution::ExecutionRequest<'request, 'authority>) -> Result<T, E>,
    ) -> Result<
        Result<(T, WorthQueryManagedIndexCallbackProof), E>,
        WorthQueryManagedIndexSessionDenial,
    > {
        self.check_current()?;
        // A failed callback cannot publish a root. Preserve its real first stop;
        // only a successful callback can proceed through the final source gate.
        match callback(&self.execution) {
            Err(cause) => Ok(Err(cause)),
            Ok(value) => {
                self.check_current()?;
                Ok(Ok((
                    value,
                    WorthQueryManagedIndexCallbackProof {
                        session: self.identity,
                        plan: self.plan.identity(),
                    },
                )))
            }
        }
    }

    pub(in crate::domain_computation) fn complete(
        self,
        proof: WorthQueryManagedIndexCallbackProof,
    ) -> Result<
        WorthQueryManagedIndexSelectionProof<'a, 'request, 'authority>,
        WorthQueryManagedIndexSessionDenial,
    > {
        use WorthQueryManagedIndexSessionDenial as Denial;
        self.check_current()?;
        if proof.session != self.identity || proof.plan != self.plan.identity() {
            return Err(Denial::AuthorityMismatch);
        }
        let (plan, release) = self
            .plan
            .complete_managed_index()
            .ok_or(Denial::TerminalReleaseMismatch)?;
        if release.scope() != worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope::GraphWork
            || release.released_reservation_count() != 1 {
            return Err(Denial::TerminalReleaseMismatch);
        }
        Ok(WorthQueryManagedIndexSelectionProof {
            execution: self.execution,
            parent: self.parent,
            request: self.request,
            association: self.association,
            source: self.source,
            session: self.identity,
            managed_run: self.managed_run,
            plan,
            release,
        })
    }
}
impl<'request, 'authority> WorthQueryManagedIndexSelectionProof<'_, 'request, 'authority> {
    pub(in crate::domain_computation) fn execution(
        &self,
    ) -> worth_execution::ExecutionRequest<'request, 'authority> {
        self.execution
    }
    pub(in crate::domain_computation) fn consult(
        &self,
    ) -> Result<(), WorthQueryManagedIndexSessionDenial> {
        self.execution
            .consult()
            .map_err(WorthQueryManagedIndexSessionDenial::Request)?;
        match self.request.interruption() {
            Some(stop) => Err(WorthQueryManagedIndexSessionDenial::Interrupted(stop)),
            None => Ok(()),
        }
    }
    pub(in crate::domain_computation) fn admits(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        source: &Arc<WorthQueryCompletedSourceRead>,
    ) -> bool {
        std::ptr::eq(self.parent, parent)
            && self.parent.identity() == parent.identity()
            && self.request.same_request(request)
            && self.association.same_association(association)
            && Arc::ptr_eq(&self.source, source)
    }
}
