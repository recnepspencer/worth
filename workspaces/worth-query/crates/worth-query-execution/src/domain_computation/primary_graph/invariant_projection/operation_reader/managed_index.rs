//! Committed decision queries retain the actual whole-Source read in this reader.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{self, ApplicationQueryNativeAccess},
    HandlerExecutionDenial,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
type Value<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
#[derive(Debug)]
enum IndexDecisionStop {
    ContextUnavailable,
    ForeignProjection,
    Native(worth_relational::facade::mvcc::CompanionPreflightStop),
}
impl std::fmt::Display for IndexDecisionStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for IndexDecisionStop {}
pub(in crate::domain_computation::primary_graph) fn native_index_decision_cause<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a worth_relational::facade::mvcc::CompanionPreflightStop> {
    match error.downcast_ref::<IndexDecisionStop>()? {
        IndexDecisionStop::Native(cause) => Some(cause),
        _ => None,
    }
}

fn unavailable() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(IndexDecisionStop::ContextUnavailable)
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn index_query<B>(
        &mut self,
        intent: B::Input,
    ) -> Result<Vec<Value<Schema, B>>, HandlerExecutionDenial>
    where
        B: ApplicationQueryBinding<Schema>,
        B::Input: ApplicationQueryIntent<Schema, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<Schema, B::PrincipalIdentity>,
        Value<Schema, B>: application_query::WorthQueryApplicationProjection<Schema, B::Query>,
    {
        let result = (|| {
            let access = self.source_query.ok_or_else(unavailable)?;
            let context = self.source_context_use.ok_or_else(unavailable)?;
            let admission = self.admission_identity.ok_or_else(unavailable)?;
            let contracts = self
                .admitted_contracts
                .as_ref()
                .ok_or_else(unavailable)?
                .clone();
            self.admit_decision_target(&worth_query_installation::facade::ApplicationOperationDecisionReadTarget::ManagedIndex {
                parent: worth_query_declaration::facade::application_query::ApplicationQueryBindingReference::of::<Schema, B>(),
            }).map_err(HandlerExecutionDenial::new)?;
            let scope = self
                .operation_scope
                .as_ref()
                .ok_or_else(unavailable)?
                .clone();
            let installed = access
                .application
                .installed_schema()
                .installed_query_binding::<B>()
                .map_err(HandlerExecutionDenial::new)?;
            // A named Query is not permission to read its fields as this operation.
            // Admit the actual installed Parent projection under its normal read
            // targets; the independent Source query still performs its own READ.
            for field in installed.query().read_graph().projections() {
                self.admit_decision_target(&worth_query_installation::facade::ApplicationOperationDecisionReadTarget::Field {
                    entity: field.entity().to_owned(), aspect: field.aspect().to_owned(), field: field.field().to_owned(),
                }).map_err(HandlerExecutionDenial::new)?;
            }
            let selected = access.shared.selected();
            let original = self
                .reader
                .native
                .committed_runtime()
                .ok_or_else(unavailable)?
                .read_truth()
                .positioned_snapshot(self.reader.snapshot)
                .map_err(|_| HandlerExecutionDenial::new(IndexDecisionStop::ForeignProjection))?;
            let borrowed = self
                .reader
                .native
                .committed_runtime()
                .ok_or_else(unavailable)?
                .read_truth()
                .positioned_snapshot(selected.application_basis().snapshot_handle())
                .map_err(|_| HandlerExecutionDenial::new(IndexDecisionStop::ForeignProjection))?;
            if original != borrowed
                || self.reader.selected_product_occurrence
                    != Some(selected.product().observation().lifecycle_incarnation())
                || self.reader.selected_product_generation
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
            let execution = self.reader.execution;
            let graph = access
                .application
                .runtime
                .primary_graph()
                .ok_or_else(unavailable)?;
            let mut native = ApplicationQueryNativeAccess::borrowed(
                graph,
                self.reader
                    .native
                    .committed_runtime_mut()
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
            self.decision_facts
                .record_managed_source(
                    &observation,
                    admission,
                    contracts,
                    self.reader.snapshot,
                    execution,
                )
                .map_err(|cause| HandlerExecutionDenial::new(IndexDecisionStop::Native(cause)))?;
            Ok(rows)
        })();
        if result.is_err() {
            self.decision_facts.fail_managed_source();
        }
        result
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;
    #[test]
    fn handler_denial_preserves_native_managed_index_decision_cause() {
        let denial = HandlerExecutionDenial::new(IndexDecisionStop::Native(
            worth_relational::facade::mvcc::CompanionPreflightStop::SelectedSourceMismatch,
        ));
        assert!(matches!(
            denial.native_decision_refusal(),
            Some(worth_relational::facade::mvcc::CompanionPreflightStop::SelectedSourceMismatch)
        ));
    }
}
