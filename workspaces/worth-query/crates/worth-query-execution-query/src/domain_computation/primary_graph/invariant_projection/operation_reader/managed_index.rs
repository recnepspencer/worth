//! Ordinary declared index queries inside an admitted decision reader.

use super::*;
use crate::domain_computation::primary_graph::application_query::{
    self, ApplicationQueryNativeAccess, SourceQueryReaderAccess,
};
use crate::domain_computation::primary_graph::HandlerExecutionDenial;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution_lineage::facade::ports::InvariantProjectionReaderInternalPort as _;
use worth_query_execution_lineage::facade::ports::OperationInvariantProjectionReaderInternalPort as _;
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::managed_index::{
    IndexDecisionFailure as Failure, WorthQueryComputationManagedIndexDenial,
};

type Value<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

mod basis;
mod scope_authority;

const fn unavailable() -> Failure {
    Failure::ContextUnavailable
}

/// The handler's door: each refusal keeps the exact boxed cause it had.
pub fn index_query<Schema: ApplicationSchema, Operation, B>(
    reader_self: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        '_,
        '_,
        Schema,
        Operation,
    >,
    source_query: Option<&SourceQueryReaderAccess<'_, Schema>>,
    intent: B::Input,
) -> Result<Vec<Value<Schema, B>>, HandlerExecutionDenial>
where
    B: ApplicationQueryBinding<Schema>,
    B::Input: ApplicationQueryIntent<Schema, Binding = B>,
    B::ScopeBinding: ApplicationQueryScopeResolution<Schema, B::PrincipalIdentity>,
    Value<Schema, B>: application_query::WorthQueryApplicationProjection<Schema, B::Query>,
{
    decision_index_query::<Schema, Operation, B>(reader_self, source_query, intent)
        .map_err(Failure::into_handler)
}

/// A partitioned computation's owner call makes the same issued decision
/// read. The read's facts are recorded with that call, and its refusal is
/// retained as a typed, comparable computation read denial.
pub(in crate::domain_computation::primary_graph) fn computation_index_query<
    Schema: ApplicationSchema,
    Operation,
    B,
>(
    reader_self: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        '_,
        '_,
        Schema,
        Operation,
    >,
    source_query: Option<&SourceQueryReaderAccess<'_, Schema>>,
    intent: B::Input,
) -> Result<Vec<Value<Schema, B>>, WorthQueryComputationManagedIndexDenial>
where
    B: ApplicationQueryBinding<Schema>,
    B::Input: ApplicationQueryIntent<Schema, Binding = B>,
    B::ScopeBinding: ApplicationQueryScopeResolution<Schema, B::PrincipalIdentity>,
    Value<Schema, B>: application_query::WorthQueryApplicationProjection<Schema, B::Query>,
{
    if let Some(context) = reader_self.source_context_use() {
        context.set(context.get().source_query_permission());
    }
    decision_index_query::<Schema, Operation, B>(reader_self, source_query, intent)
        .map_err(Failure::into_computation)
}

/// The issued decision read: declared Parent and projected fields first, then
/// the read basis (the committed admitted basis, or an applied candidate's
/// aligned before basis), then the resolved scope's declared authority, all
/// before any provider call. The complete managed Source fact of every read,
/// per scope, is recorded with the owning decision or computation call.
fn decision_index_query<Schema: ApplicationSchema, Operation, B>(
    reader_self: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        '_,
        '_,
        Schema,
        Operation,
    >,
    source_query: Option<&SourceQueryReaderAccess<'_, Schema>>,
    intent: B::Input,
) -> Result<Vec<Value<Schema, B>>, Failure>
where
    B: ApplicationQueryBinding<Schema>,
    B::Input: ApplicationQueryIntent<Schema, Binding = B>,
    B::ScopeBinding: ApplicationQueryScopeResolution<Schema, B::PrincipalIdentity>,
    Value<Schema, B>: application_query::WorthQueryApplicationProjection<Schema, B::Query>,
{
    let result = (|| {
        let access = source_query.ok_or_else(unavailable)?;
        let context = reader_self.source_context_use().ok_or_else(unavailable)?;
        let admission = reader_self.admission_identity().ok_or_else(unavailable)?;
        let contracts = reader_self
            .admitted_contracts()
            .cloned()
            .ok_or_else(unavailable)?;
        let authority = scope_authority::DeclaredScopeAuthority::admit::<Schema, Operation, B>(
            reader_self,
            &contracts,
        )?;
        let installed = access
            .application
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(Failure::Installation)?;
        for field in installed.query().read_graph().projections() {
            reader_self
                .admit_decision_target(
                    &worth_query_installation::facade::ApplicationOperationDecisionReadTarget::Field {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                    },
                )
                .map_err(Failure::Undeclared)?;
        }
        let basis = basis::DecisionReadBasis::admit(reader_self, access)?;
        let execution = reader_self.reader().execution();
        let mut native = basis.native_access(access)?;
        let (rows, observation) = application_query::managed_index::read_decision::<Schema, B>(
            execution,
            access,
            intent,
            context,
            &mut |scope, principal| authority.admit_scope(reader_self, scope, principal),
            &mut native,
            basis.applied(),
        )
        .map_err(Failure::ManagedIndex)?;
        let snapshot = reader_self.reader().snapshot();
        reader_self
            .decision_facts_mut()
            .record_managed_source(&observation, admission, contracts, snapshot, execution)
            .map_err(Failure::Native)?;
        Ok(rows)
    })();
    if result.is_err() {
        reader_self.decision_facts_mut().fail_managed_source();
    }
    result
}
