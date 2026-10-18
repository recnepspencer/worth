//! One Query pipeline's access to its original native runtime.
//!
//! This is borrow transport, never permission. All query/session, principal,
//! scope, disclosure and selected-basis admission still runs at its owner.

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraph;
use worth_relational::facade::runtime::RelationalRuntime;

pub(in crate::domain_computation::primary_graph) enum ApplicationQueryNativeAccess<'a> {
    Acquire,
    Borrowed(&'a mut RelationalRuntime),
}

impl<'a> ApplicationQueryNativeAccess<'a> {
    pub(in crate::domain_computation::primary_graph) fn acquire() -> Self {
        Self::Acquire
    }

    pub(in crate::domain_computation::primary_graph) fn borrowed(
        graph: &WorthQueryPrimaryGraph,
        runtime: &'a mut RelationalRuntime,
    ) -> Result<Self, super::WorthQueryApplicationQueryAdmissionDenial> {
        if graph.relational_runtime_instance_id()
            != runtime.main_branch_identity().runtime_instance_id()
        {
            return Err(super::WorthQueryApplicationQueryAdmissionDenial::new(
                super::WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                "original native source read",
            ));
        }
        Ok(Self::Borrowed(runtime))
    }

    pub(in crate::domain_computation::primary_graph) fn with_runtime_mut<T, E>(
        &mut self,
        graph: &WorthQueryPrimaryGraph,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        foreign: impl FnOnce() -> E,
        read: impl FnOnce(&mut RelationalRuntime) -> Result<T, E>,
    ) -> Result<T, E> {
        match self {
            Self::Acquire => graph.with_runtime_mut(execution, read),
            Self::Borrowed(runtime) => {
                if graph.relational_runtime_instance_id()
                    != runtime.main_branch_identity().runtime_instance_id()
                {
                    return Err(foreign());
                }
                read(runtime)
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn execute_query_read<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
        basis: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
        read: impl FnOnce(
            &mut RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<
        (
            T,
            crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
        ),
        crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial,
    > {
        match self {
            Self::Acquire => work.execute_query_read(execution, basis, read),
            Self::Borrowed(runtime) => work.execute_query_read_borrowed(basis, runtime, read),
        }
    }
}
