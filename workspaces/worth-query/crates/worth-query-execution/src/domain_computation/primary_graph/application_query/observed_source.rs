use std::marker::PhantomData;

use worth_foundational::facade::{AspectContractRevision, AspectKey, CanonicalDigestId};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, KindId, VersionId},
};

use super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

mod checkpoint_facts;
mod denial;
mod footprint_accounting;
pub(in crate::domain_computation::primary_graph) mod source_identity;
pub use denial::{WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind};
pub(in crate::domain_computation::primary_graph) use source_identity::WorthQueryObservedSourceEpoch;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryObservedAspectRevision {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) aspect: AspectKey,
    pub(in crate::domain_computation::primary_graph) contract_revision: AspectContractRevision,
    pub(in crate::domain_computation::primary_graph) native_revision: Option<u64>,
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
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedAspectRevision>,
    pub(in crate::domain_computation::primary_graph) adjacencies:
        Vec<WorthQueryObservedAdjacencyRevision>,
}

/// Opaque proof of the native source projected into one public query row.
/// It is descriptive input to a later governed mutation, not read authority.
pub struct WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation) runtime_authority: u64,
    pub(in crate::domain_computation) schema_binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation) query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(in crate::domain_computation) parameter_binding_identity: CanonicalDigestId,
    pub(in crate::domain_computation) query_identifier: String,
    pub(in crate::domain_computation) branch: BranchId,
    pub(in crate::domain_computation) selection: WorthQueryApplicationBasisSelectionIdentity,
    pub(in crate::domain_computation) model_root: EntityId,
    pub(super) source_meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
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
        Self {
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            query_identity: self.query_identity.clone(),
            parameter_binding_identity: self.parameter_binding_identity,
            query_identifier: self.query_identifier.clone(),
            branch: self.branch.clone(),
            selection: self.selection.clone(),
            model_root: self.model_root,
            source_meaning: std::sync::Arc::clone(&self.source_meaning),
            _marker: PhantomData,
        }
    }
}

impl<Query> WorthQueryObservedSource<Query> {
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

    /// Content-stable source commitment retained from the immutable basis.
    /// Runtime and branch affinity are validated before durable idempotency.
    pub(in crate::domain_computation) fn idempotency_identity(&self) -> [u8; 32] {
        self.source_meaning.durable_idempotency_identity()
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
    pub(in crate::domain_computation) fn validate_and_into_facts(
        self,
        runtime_authority: u64,
        binding: &ApplicationSchemaBindingIdentity,
        branch: &BranchId,
        model_root: EntityId,
        selected_product: &crate::basis::WorthQueryProductBranchReadIdentity,
        expected_query_identifier: &str,
        expected_query_identity: &WorthQueryInstalledApplicationQueryIdentity,
        layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    ) -> Result<
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        WorthQuerySourceExpectationDenial,
    > {
        use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
        use WorthQuerySourceExpectationDenialKind as Kind;

        if self.runtime_authority != runtime_authority {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignApplication,
                expected_query_identifier,
            ));
        }
        if self.schema_binding.runtime_ordinal() != binding.runtime_ordinal()
            || self.schema_binding.generation() != binding.generation()
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignInstallation,
                expected_query_identifier,
            ));
        }
        if self.schema_binding.schema_identity() != binding.schema_identity()
            || self.schema_binding.package_identity() != binding.package_identity()
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignSchema,
                expected_query_identifier,
            ));
        }
        if self.query_identifier != expected_query_identifier {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::SourceContractMismatch,
                expected_query_identifier,
            ));
        }
        if &self.query_identity != expected_query_identity {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::SourceContractMismatch,
                expected_query_identifier,
            ));
        }
        let WorthQueryApplicationBasisSelectionIdentity::Product(observed_product) =
            &self.selection
        else {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignBranch,
                expected_query_identifier,
            ));
        };
        if &self.branch != branch || !selected_product.same_branch_occurrence(observed_product) {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignBranch,
                expected_query_identifier,
            ));
        }
        if self.model_root != model_root {
            return Err(WorthQuerySourceExpectationDenial::new(
                Kind::ForeignModel,
                expected_query_identifier,
            ));
        }
        self.validate_completeness(expected_query_identifier)?;
        let footprint = self.source_meaning.footprint().clone();
        let mut facts = Vec::with_capacity(
            footprint
                .entities
                .len()
                .saturating_add(footprint.aspects.len())
                .saturating_add(footprint.adjacencies.len()),
        );
        facts.extend(
            footprint
                .entities
                .into_iter()
                .map(|entity_id| Fact::SourceEntity { entity_id }),
        );
        for aspect in footprint.aspects {
            layout
                .aspect_contract(&aspect.entity_name, &aspect.aspect)
                .filter(|contract| contract.revision() == aspect.contract_revision)
                .ok_or_else(|| {
                    WorthQuerySourceExpectationDenial::new(
                        Kind::SourceContractMismatch,
                        aspect.aspect.as_str(),
                    )
                })?;
            facts.push(Fact::SourceAspectRevision {
                entity_id: aspect.entity,
                aspect: aspect.aspect,
                native_revision: aspect.native_revision,
            });
        }
        facts.extend(footprint.adjacencies.into_iter().map(|adjacency| {
            Fact::SourceAdjacencyRevision {
                relation_kind: adjacency.relation_kind,
                anchor: adjacency.anchor,
                direction: adjacency.direction,
                native_revision: adjacency.native_revision,
                comparison_work_limit: adjacency.comparison_work_limit,
                endpoints: adjacency.endpoints,
            }
        }));
        Ok(facts)
    }
}

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn bind_application_source_expectation<Binding, Scope>(
        &self,
        admission: &mut crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    ) -> Result<[u8; 32], WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let expected_query_identifier = <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    Binding::IDENTITY,
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignApplication,
                Binding::IDENTITY,
            )
        })?;
        let expected_query_identity = self
            .installed_schema
            .installed_query_identity_by_name(expected_query_identifier)
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    expected_query_identifier,
                )
            })?;
        let selected_product = admission
            .graph_work()
            .mutation_product()
            .map(|product| {
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    product.observation(),
                )
            })
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::ForeignBranch,
                    Binding::IDENTITY,
                )
            })?;
        let idempotency_identity = source.idempotency_identity();
        let partition_identity = source.partition_identity();
        let facts = source.validate_and_into_facts(
            self.runtime.authority_identity().as_u64(),
            &self.installed_schema.binding_identity(),
            admission.graph_work_branch(),
            admission.scope_entity_id(),
            &selected_product,
            expected_query_identifier,
            expected_query_identity,
            &graph.layout,
        )?;
        admission.bind_source_partition(partition_identity);
        admission.bind_source_facts(facts);
        Ok(idempotency_identity)
    }
}
