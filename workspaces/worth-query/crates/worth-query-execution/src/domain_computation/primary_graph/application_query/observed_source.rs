use std::marker::PhantomData;

use worth_foundational::facade::{AspectContractRevision, AspectKey, CanonicalDigestId, FieldKey};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, KindId, VersionId},
};

use super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

mod admitted_clone;
mod admitted_epoch;
mod denial;
mod fact_conversion;
mod footprint_accounting;
mod prepared_expectation;
mod result_set;
mod retained_parameters;
mod root_selection;
mod scope_selector;
pub(in crate::domain_computation::primary_graph) use admitted_clone::WorthQueryObservedSourceCloneStop;
pub(in crate::domain_computation::primary_graph) use admitted_epoch::WorthQueryObservedEpochStop;
pub use denial::{WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind};
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
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        idempotency
            .bind_source(Some(&self.identity))
            .bind_source_partition(&self.partition_identity)
    }
}
pub(in crate::domain_computation::primary_graph) mod source_identity;
pub(in crate::domain_computation::primary_graph::application_query) use retained_parameters::WorthQueryRetainedObservedParameters;
pub(in crate::domain_computation::primary_graph) use root_selection::WorthQueryObservedRootSelection;
pub(in crate::domain_computation::primary_graph) use scope_selector::WorthQueryObservedScopeSelector;
pub(in crate::domain_computation::primary_graph) use source_identity::{
    WorthQueryCheckpointSourceIdentity, WorthQueryObservedSourceEpoch,
    WorthQueryObservedSourceSelection, WorthQueryRuntimeSourceIdentity,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryObservedFieldRevision {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) aspect: AspectKey,
    pub(in crate::domain_computation::primary_graph) field: FieldKey,
    pub(in crate::domain_computation::primary_graph) contract_revision: AspectContractRevision,
    pub(in crate::domain_computation::primary_graph) native_revision:
        Option<worth_relational::facade::runtime::RelationalFieldRevision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryObservedAdjacencyRevision {
    pub(in crate::domain_computation::primary_graph) anchor: EntityId,
    pub(in crate::domain_computation::primary_graph) relation_kind: KindId,
    pub(in crate::domain_computation::primary_graph) direction:
        worth_relational::facade::runtime::RelationalAdjacencyDirection,
    pub(in crate::domain_computation::primary_graph) native_revision: Option<VersionId>,
    pub(in crate::domain_computation::primary_graph) comparison_work_limit: usize,
    pub(in crate::domain_computation::primary_graph) endpoints: Vec<EntityId>,
}

impl Ord for WorthQueryObservedAdjacencyRevision {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.anchor,
            self.relation_kind,
            adjacency_direction_rank(self.direction),
            self.native_revision,
            self.comparison_work_limit,
            &self.endpoints,
        )
            .cmp(&(
                other.anchor,
                other.relation_kind,
                adjacency_direction_rank(other.direction),
                other.native_revision,
                other.comparison_work_limit,
                &other.endpoints,
            ))
    }
}

impl PartialOrd for WorthQueryObservedAdjacencyRevision {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

const fn adjacency_direction_rank(
    direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
) -> u8 {
    match direction {
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph) root: EntityId,
    pub(in crate::domain_computation::primary_graph) complete: bool,
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedFieldRevision>,
    pub(in crate::domain_computation::primary_graph) adjacencies:
        Vec<WorthQueryObservedAdjacencyRevision>,
    pub(in crate::domain_computation::primary_graph) root_selection:
        Option<std::sync::Arc<WorthQueryObservedRootSelection>>,
}

/// Opaque proof of the native source projected into one public query row.
/// It is descriptive input to a later governed mutation, not read authority.
pub struct WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation) runtime_authority: u64,
    pub(in crate::domain_computation) schema_binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation) query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(in crate::domain_computation) parameter_binding_identity: CanonicalDigestId,
    pub(super) parameters: std::sync::Arc<WorthQueryRetainedObservedParameters>,
    pub(in crate::domain_computation) query_identifier: std::sync::Arc<String>,
    pub(in crate::domain_computation) branch: std::sync::Arc<BranchId>,
    pub(in crate::domain_computation) selection: WorthQueryApplicationBasisSelectionIdentity,
    pub(in crate::domain_computation) model_root: EntityId,
    pub(super) source_meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
    pub(super) scope_selector: std::sync::Arc<WorthQueryObservedScopeSelector>,
    pub(super) _descriptor_charge:
        std::sync::Arc<super::resource_lifecycle::WorthQueryRetainedSourceCharge>,
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

    pub(in crate::domain_computation::primary_graph) fn output_source_epoch(
        &self,
    ) -> Option<source_identity::WorthQueryObservedSourceEpoch> {
        source_identity::WorthQueryObservedSourceEpoch::from_observation(
            self.query_identity.as_bytes(),
            self.parameter_binding_identity.bytes(),
            self.source_meaning.footprint().root,
            &self.selection,
            std::sync::Arc::clone(&self.source_meaning),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn source_root(&self) -> EntityId {
        self.source_meaning.footprint().root
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn footprint_for_test(
        &self,
    ) -> &WorthQueryObservedSourceFootprint {
        self.source_meaning.footprint()
    }
}
