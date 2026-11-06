use std::marker::PhantomData;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::{history::BranchId, identity::EntityId};

use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;

pub(in crate::domain_computation::primary_graph::application_query) mod comparison;
mod denial;
mod expression_context;
pub(crate) mod fact_conversion;
pub(in crate::domain_computation::primary_graph) use fact_conversion::SourceFactExpansionControl;
mod fact_sources;
mod footprint_accounting;
pub(crate) mod prepared_expectation;
mod query_context_binding;
pub(in crate::domain_computation) use query_context_binding::ObservedQueryExpressionContext;
mod realized_footprint;
mod result_set;
pub(crate) mod retained_dependencies;
pub(crate) mod retained_parameters;
pub(crate) mod root_selection;
mod scope_selector;
mod workflow_capture;
pub use denial::WorthQuerySourceExpectationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::observed_source_denial::WorthQuerySourceExpectationDenialKind;
pub(in crate::domain_computation::primary_graph::application_query) use expression_context::RetainedExpressionContext;
pub use prepared_expectation::WorthQueryPendingSourceExpectation;
pub(in crate::domain_computation::primary_graph) use prepared_expectation::{
    BoundStableObservedSourceFacts, PreparedObservedSourceExpectation,
};
pub use result_set::WorthQueryObservedResultSet;

/// An observed query source accepted as an admitted mutation's source expectation.
///
/// Pass it to `bind_idempotency` so the idempotency binding covers that source.
#[derive(Clone, Copy)]
pub struct WorthQueryBoundSourceExpectation {
    kind: WorthQuerySourceExpectationKind,
    identity: [u8; 32],
    partition_identity: [u8; 32],
}

impl WorthQueryBoundSourceExpectation {
    pub(in crate::domain_computation::primary_graph) const fn recorded_source_identity(
        self,
    ) -> WorthQueryRuntimeSourceIdentity {
        WorthQueryRuntimeSourceIdentity::new(self.identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn partition_identity(self) -> [u8; 32] {
        self.partition_identity
    }

    pub fn bind_idempotency(
        self,
        idempotency: crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding{
        idempotency
            .bind_source(Some(&self.identity))
            .bind_source_partition(&self.partition_identity)
    }
}
pub(crate) mod source_identity;
pub(in crate::domain_computation::primary_graph) use crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector;
pub(in crate::domain_computation::primary_graph::application_query) use retained_parameters::WorthQueryRetainedObservedParameters;
pub(in crate::domain_computation::primary_graph) use root_selection::WorthQueryObservedRootSelection;
pub(in crate::domain_computation::primary_graph::application_query) use root_selection::{
    DeclaredComponentSelection, DeclaredManagedIndexSelection, DeclaredOrbitSelection,
    ManagedSourceDependencies,
};
pub(in crate::domain_computation::primary_graph) use {source_identity::WorthQueryObservedSourceEpoch, source_identity::WorthQueryObservedSourceSelection};
use crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity;

pub(in crate::domain_computation) use super::super::source_facts::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph) root: EntityId,
    pub(in crate::domain_computation::primary_graph) complete: bool,
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedFieldRevision>,
    pub adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
    pub root_selection: Option<std::sync::Arc<WorthQueryObservedRootSelection>>,
}

/// Opaque proof of the native source projected into one public query row.
/// It is descriptive input to a later governed mutation, not read authority.
pub struct WorthQueryObservedSource<Query> {
    pub(super) expression_context: Option<std::sync::Arc<RetainedExpressionContext>>,
    pub(in crate::domain_computation) runtime_authority: u64,
    pub(in crate::domain_computation) schema_binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation) query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(in crate::domain_computation) parameter_binding_identity: CanonicalDigestId,
    pub(super) parameters: std::sync::Arc<WorthQueryRetainedObservedParameters>,
    pub(in crate::domain_computation) query_identifier: std::sync::Arc<String>,
    pub(in crate::domain_computation) branch: std::sync::Arc<BranchId>,
    pub(in crate::domain_computation) selection: WorthQueryApplicationBasisSelectionIdentity,
    pub(in crate::domain_computation) model_root: EntityId,
    // The current invocation's actual Source custody cannot be replaced by an
    // interned semantic Meaning retaining an older admitted footprint.
    pub(super) invocation_source_authorization: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization,
        >,
    >,
    pub(super) invocation_managed_source: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
        >,
    >,
    pub(super) source_meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
    pub(super) scope_selector: std::sync::Arc<WorthQueryObservedScopeSelector>,

    pub(in crate::domain_computation) _marker: PhantomData<fn() -> Query>,
}

impl<Query> std::fmt::Debug for WorthQueryObservedSource<Query> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryObservedSource")
            .finish_non_exhaustive()
    }
}

impl<Query> Clone for WorthQueryObservedSource<Query> {
    fn clone(&self) -> Self {
        self.retyped()
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    /// The immutable native row and its exact expression schema, if requested
    /// during the read. Operand `source` is a record in declared projection
    /// order. This is descriptive data, never read or mutation permission.
    pub fn expression_context(
        &self,
    ) -> Option<(
        &worth_foundational::expression_api::ExpressionSchema,
        &worth_foundational::expression_api::ExpressionValue,
    )> {
        self.expression_context
            .as_ref()
            .map(|context| (context.schema(), context.value()))
    }

    #[doc(hidden)]
    pub fn root_entity_for_test(&self) -> EntityId {
        self.model_root
    }

    pub(in crate::domain_computation::primary_graph) fn has_complete_output_dependencies(
        &self,
    ) -> bool {
        let footprint = self.source_meaning.footprint();
        footprint.complete && footprint.root_selection.is_some()
    }

    pub(in crate::domain_computation) fn partition_identity(&self) -> [u8; 32] {
        // The installed source expectation fixes the query contract before
        // this enters output lineage. Its admitted parameter identity is the
        // exact varying partition coordinate.
        *self.parameter_binding_identity.bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn selected_product_commit(
        &self,
    ) -> Option<&worth_runtime_world::facade::CompositeCommitIdentity> {
        match &self.selection {
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                Some(product.selected_commit())
            }
            WorthQueryApplicationBasisSelectionIdentity::Relational => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn selected_product_occurrence(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchIncarnation> {
        match &self.selection {
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                Some(product.lifecycle_incarnation())
            }
            WorthQueryApplicationBasisSelectionIdentity::Relational => None,
        }
    }

    pub(in crate::domain_computation) fn validate_completeness(
        &self,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        self.source_meaning
            .footprint()
            .complete
            .then_some(())
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
                    subject,
                )
            })
    }

    pub fn output_source_epoch(&self) -> Option<source_identity::WorthQueryObservedSourceEpoch> {
        source_identity::WorthQueryObservedSourceEpoch::from_observation(
            self.query_identity.as_bytes(),
            self.parameter_binding_identity.bytes(),
            self.source_meaning.footprint().root,
            &self.selection,
            std::sync::Arc::clone(&self.source_meaning),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn checkpoint_observation_identity(
        &self,
    ) -> [u8; 32] {
        self.source_meaning.checkpoint_identity().bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn source_root(&self) -> EntityId {
        self.source_meaning.footprint().root
    }

    pub fn footprint_for_test(&self) -> &WorthQueryObservedSourceFootprint {
        self.source_meaning.footprint()
    }
}

mod court_doors_for_test;
use crate::domain_computation::vocabulary::primary_graph::observed_source::WorthQuerySourceExpectationKind;
pub use prepared_expectation::PreparedExpectationOps;
pub(in crate::domain_computation::primary_graph) use prepared_expectation::PreparedExpectationScopedOps;
