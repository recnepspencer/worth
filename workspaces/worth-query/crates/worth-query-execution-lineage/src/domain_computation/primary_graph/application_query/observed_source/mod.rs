//! Pure observed source structs, footprints, identities, and expression contexts.

use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::{history::BranchId, identity::EntityId};

use super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

pub mod applied_publication;
pub mod comparison;
pub mod denial;
pub mod expression_context;
pub mod fact_conversion;
pub mod fact_sources;
pub mod footprint_accounting;
pub mod realized_footprint;
pub mod retained_parameters;
pub mod root_selection;
pub mod scope_selector;
pub mod source_identity;

pub use super::super::entity_resolution::WorthQueryObservedScopeSelector;
pub use super::super::source_facts::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
};
pub use applied_publication::AppliedSourcePublication;
pub use denial::{WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind};
pub use expression_context::{ObservedQueryExpressionContext, RetainedExpressionContext};
pub use retained_parameters::WorthQueryRetainedObservedParameters;
pub use root_selection::{
    DeclaredComponentSelection, DeclaredManagedIndexSelection, DeclaredOrbitSelection,
    ManagedSourceDependencies, WorthQueryObservedRootSelection,
};
pub use source_identity::{
    WorthQueryCheckpointSourceIdentity, WorthQueryObservedSourceEpoch,
    WorthQueryObservedSourceSelection, WorthQueryRuntimeSourceIdentity,
};

#[derive(Clone, Copy)]
pub struct WorthQueryBoundSourceExpectation {
    pub(crate) kind: WorthQuerySourceExpectationKind,
    pub(crate) identity: [u8; 32],
    pub(crate) partition_identity: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceExpectationKind {
    Row,
    ResultSet,
}

impl WorthQuerySourceExpectationKind {
    pub const fn code(self) -> u64 {
        match self {
            Self::Row => 1,
            Self::ResultSet => 2,
        }
    }
}

impl WorthQueryBoundSourceExpectation {
    pub const fn recorded_source_identity(self) -> WorthQueryRuntimeSourceIdentity {
        WorthQueryRuntimeSourceIdentity::new(self.identity)
    }

    pub const fn partition_identity(self) -> [u8; 32] {
        self.partition_identity
    }

    pub fn bind_idempotency(
        self,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        idempotency
            .bind_source(Some(&self.identity))
            .bind_source_partition(&self.partition_identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryObservedSourceFootprint {
    pub root: EntityId,
    pub complete: bool,
    pub entities: Vec<EntityId>,
    pub aspects: Vec<WorthQueryObservedFieldRevision>,
    pub adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
    pub root_selection: Option<Arc<WorthQueryObservedRootSelection>>,
}

pub struct WorthQueryObservedSource<Query> {
    pub(crate) expression_context: Option<Arc<RetainedExpressionContext>>,
    pub(crate) runtime_authority: u64,
    pub(crate) schema_binding: ApplicationSchemaBindingIdentity,
    pub(crate) query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(crate) parameter_binding_identity: CanonicalDigestId,
    pub(crate) parameters: Arc<WorthQueryRetainedObservedParameters>,
    pub(crate) query_identifier: Arc<String>,
    pub(crate) branch: Arc<BranchId>,
    pub(crate) selection: WorthQueryApplicationBasisSelectionIdentity,
    pub(crate) model_root: EntityId,
    pub(crate) invocation_source_authorization: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    >,
    pub(crate) invocation_managed_source: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    >,
    pub(crate) source_meaning: Arc<source_identity::WorthQueryObservedSourceMeaning>,
    pub(crate) scope_selector: Arc<WorthQueryObservedScopeSelector>,
    pub(crate) _marker: PhantomData<fn() -> Query>,
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
    pub fn retyped<Other>(&self) -> WorthQueryObservedSource<Other> {
        WorthQueryObservedSource {
            expression_context: self.expression_context.as_ref().map(Arc::clone),
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            query_identity: self.query_identity.clone(),
            parameter_binding_identity: self.parameter_binding_identity,
            parameters: Arc::clone(&self.parameters),
            query_identifier: Arc::clone(&self.query_identifier),
            branch: Arc::clone(&self.branch),
            selection: self.selection.clone(),
            model_root: self.model_root,
            invocation_source_authorization: self
                .invocation_source_authorization
                .as_ref()
                .map(Arc::clone),
            invocation_managed_source: self.invocation_managed_source.as_ref().map(Arc::clone),
            source_meaning: Arc::clone(&self.source_meaning),
            scope_selector: Arc::clone(&self.scope_selector),
            _marker: PhantomData,
        }
    }

    pub fn retain_expression_binding(&self) -> Option<ObservedQueryExpressionContext> {
        self.expression_context.as_ref().map(|context| {
            ObservedQueryExpressionContext::from_parts(
                Arc::clone(&self.query_identifier),
                Arc::clone(context),
            )
        })
    }

    pub(crate) fn retained_parameters(&self) -> &Arc<WorthQueryRetainedObservedParameters> {
        &self.parameters
    }

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

    pub fn has_complete_output_dependencies(&self) -> bool {
        let footprint = self.source_meaning.footprint();
        footprint.complete && footprint.root_selection.is_some()
    }

    pub fn partition_identity(&self) -> [u8; 32] {
        *self.parameter_binding_identity.bytes()
    }

    pub fn selected_product_commit(
        &self,
    ) -> Option<&worth_runtime_world::facade::CompositeCommitIdentity> {
        match &self.selection {
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                Some(product.selected_commit())
            }
            WorthQueryApplicationBasisSelectionIdentity::Relational => None,
        }
    }

    pub fn selected_product_occurrence(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchIncarnation> {
        match &self.selection {
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                Some(product.lifecycle_incarnation())
            }
            WorthQueryApplicationBasisSelectionIdentity::Relational => None,
        }
    }

    pub fn validate_completeness(
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

    pub fn output_source_epoch(&self) -> Option<WorthQueryObservedSourceEpoch> {
        WorthQueryObservedSourceEpoch::from_observation(
            self.query_identity.as_bytes(),
            self.parameter_binding_identity.bytes(),
            self.source_meaning.footprint().root,
            &self.selection,
            Arc::clone(&self.source_meaning),
        )
    }

    pub fn checkpoint_observation_identity(&self) -> [u8; 32] {
        self.source_meaning.checkpoint_identity().bytes()
    }

    pub fn source_root(&self) -> EntityId {
        self.source_meaning.footprint().root
    }

    pub fn footprint_for_test(&self) -> &WorthQueryObservedSourceFootprint {
        self.source_meaning.footprint()
    }

    /// A lookup key from Query's issued row source, not a scene-authored key.
    pub fn managed_derived_view_key(&self) -> WorthQueryManagedDerivedViewKey {
        WorthQueryManagedDerivedViewKey::new(
            self.model_root,
            self.source_root(),
            self.parameter_binding_identity,
        )
    }
}

/// Query-issued row identity: an exact source root and parameter binding,
/// never a consumer-selected scene key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryManagedDerivedViewKey {
    model_root: EntityId,
    root: EntityId,
    parameters: CanonicalDigestId,
}

impl WorthQueryManagedDerivedViewKey {
    pub const fn new(model_root: EntityId, root: EntityId, parameters: CanonicalDigestId) -> Self {
        Self {
            model_root,
            root,
            parameters,
        }
    }

    pub const fn model_root(&self) -> EntityId {
        self.model_root
    }

    pub const fn root(&self) -> EntityId {
        self.root
    }

    pub const fn parameters(&self) -> &CanonicalDigestId {
        &self.parameters
    }
}

mod court_doors_for_test;
