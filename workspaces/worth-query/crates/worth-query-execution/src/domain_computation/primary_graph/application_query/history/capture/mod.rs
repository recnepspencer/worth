//! Only the completed disclosed query can bind a prepared native subject selection.
use std::{marker::PhantomData, sync::Arc};

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding,
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use worth_relational::facade::{history::RelationalRecordHistorySelection, identity::EntityId};

use super::super::WorthQueryObservedResultSet;
use super::{denial::WorthQueryApplicationHistoryDenial as Denial, SubjectPlan, SubjectQuery};

mod prepare;
pub(in crate::domain_computation::primary_graph::application_query) use prepare::prepare as prepare_capture;
mod walk;

pub(in crate::domain_computation::primary_graph::application_query) struct PreparedHistoryCapture<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: CanonicalDigestId,
    scope: EntityId,
    product: crate::basis::WorthQueryProductObservationLease,
    native: Option<RelationalRecordHistorySelection>,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
    _history: PhantomData<fn() -> (Schema, History)>,
}

/// An opaque captured generation set. It retains provenance, never a future read permission.
pub struct WorthQueryCapturedApplicationHistorySubjects<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    pub(super) owner: Arc<HistorySubjectSource<Schema, History>>,
}

pub(super) struct HistorySubjectSource<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    pub(super) observed: WorthQueryObservedResultSet<SubjectQuery<Schema, History>>,
    // The captured subject set keeps its original Product root pinned until drop.
    _product: crate::basis::WorthQueryProductObservationLease,
    pub(super) native: Option<RelationalRecordHistorySelection>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>> Clone
    for WorthQueryCapturedApplicationHistorySubjects<Schema, History>
{
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
        }
    }
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>
    PreparedHistoryCapture<Schema, History>
{
    /// The caller supplies only this same successful completion's owner-issued source.
    pub(in crate::domain_computation::primary_graph::application_query) fn bind(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        observed: &WorthQueryObservedResultSet<SubjectQuery<Schema, History>>,
    ) -> Result<WorthQueryCapturedApplicationHistorySubjects<Schema, History>, Denial> {
        let source = observed.source();
        let same_root = match &source.selection {
            super::super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity::Product(identity) => {
                identity.matches_observation(self.product.observation())
            }
            super::super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity::Relational => false,
        };
        if source.runtime_authority != self.runtime
            || source.schema_binding != self.schema
            || source.query_identity != self.query
            || source.parameter_binding_identity != self.parameters
            || source.model_root != self.scope
            || !same_root
        {
            return Err(Denial::SubjectMismatch);
        }
        let (captured, _) = execution
            .run_native(|_| WorthQueryCapturedApplicationHistorySubjects {
                owner: Arc::new(HistorySubjectSource {
                    observed: observed.clone(),
                    _product: self.product,
                    native: self.native,
                    _memory: self.memory,
                }),
            })
            .map_err(Denial::Request)?;
        Ok(captured)
    }

    fn owner_bytes() -> usize {
        size_of::<HistorySubjectSource<Schema, History>>() + 2 * size_of::<usize>()
    }

    fn identity(
        plan: &SubjectPlan<'_, Schema, History>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            runtime: plan.runtime_authority.as_u64(),
            schema: plan.query.binding_identity().clone(),
            query: plan.query.identity().clone(),
            parameters: *plan.parameters.identity(),
            scope: plan.scope.entity_id(),
            product: plan.basis.retained_product(),
            native: None,
            memory,
            _history: PhantomData,
        }
    }
}
