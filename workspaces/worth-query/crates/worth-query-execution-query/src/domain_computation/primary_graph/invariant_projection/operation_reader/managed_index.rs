//! Ordinary declared index queries inside an admitted decision reader.

use worth_query_execution_lineage::facade::ports::InvariantProjectionReaderInternalPort as _;
use worth_query_execution_lineage::facade::ports::OperationInvariantProjectionReaderInternalPort as _;
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
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::managed_index::IndexDecisionStop;

type Value<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

fn unavailable() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(IndexDecisionStop::ContextUnavailable)
}

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
    let result = (|| {
        let access = source_query.ok_or_else(unavailable)?;
        let context = reader_self.source_context_use().ok_or_else(unavailable)?;
        let admission = reader_self.admission_identity().ok_or_else(unavailable)?;
        let contracts = reader_self
            .admitted_contracts()
            .cloned()
            .ok_or_else(unavailable)?;
        reader_self
            .admit_decision_target(
                &worth_query_installation::facade::ApplicationOperationDecisionReadTarget::ManagedIndex {
                    parent: worth_query_declaration::facade::application_query::ApplicationQueryBindingReference::of::<Schema, B>(),
                },
            )
            .map_err(HandlerExecutionDenial::new)?;
        let scope = reader_self
            .operation_scope()
            .as_ref()
            .ok_or_else(unavailable)?
            .clone();
        let installed = access
            .application
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(HandlerExecutionDenial::new)?;
        for field in installed.query().read_graph().projections() {
            reader_self
                .admit_decision_target(
                    &worth_query_installation::facade::ApplicationOperationDecisionReadTarget::Field {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                    },
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        let selected = access.shared.selected();
        let original = reader_self
            .reader()
            .native()
            .committed_runtime()
            .ok_or_else(unavailable)?
            .read_truth()
            .positioned_snapshot(reader_self.reader().snapshot())
            .map_err(|_| HandlerExecutionDenial::new(IndexDecisionStop::ForeignProjection))?;
        let borrowed = reader_self
            .reader()
            .native()
            .committed_runtime()
            .ok_or_else(unavailable)?
            .read_truth()
            .positioned_snapshot(selected.application_basis().snapshot_handle())
            .map_err(|_| HandlerExecutionDenial::new(IndexDecisionStop::ForeignProjection))?;
        if original != borrowed
            || reader_self.reader().selected_product_occurrence()
                != Some(selected.product().observation().lifecycle_incarnation())
            || reader_self.reader().selected_product_generation()
                != Some(
                    selected
                        .product()
                        .observation()
                        .reference_generation()
                        .get(),
                )
        {
            return Err(HandlerExecutionDenial::new(
                IndexDecisionStop::ForeignProjection,
            ));
        }
        let execution = reader_self.reader().execution();
        let graph = access
            .application
            .runtime
            .primary_graph()
            .ok_or_else(unavailable)?;
        let mut native = ApplicationQueryNativeAccess::borrowed(
            graph,
            reader_self
                .reader()
                .native()
                .committed_runtime()
                .ok_or_else(unavailable)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
        let (rows, observation) = application_query::managed_index::read_decision::<Schema, B>(
            execution,
            access,
            intent,
            context,
            &scope,
            &mut native,
        )
        .map_err(HandlerExecutionDenial::new)?;
        let snapshot = reader_self.reader().snapshot();
        reader_self
            .decision_facts_mut()
            .record_managed_source(&observation, admission, contracts, snapshot, execution)
            .map_err(|cause| HandlerExecutionDenial::new(IndexDecisionStop::Native(cause)))?;
        Ok(rows)
    })();
    if result.is_err() {
        reader_self.decision_facts_mut().fail_managed_source();
    }
    result
}
