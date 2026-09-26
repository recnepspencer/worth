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

mod denial;
mod fact_conversion;
mod footprint_accounting;
mod result_set;
mod root_selection;
pub use denial::{WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind};
pub use result_set::WorthQueryObservedResultSet;

pub struct WorthQueryBoundSourceExpectation {
    identity: [u8; 32],
    partition_identity: [u8; 32],
}

impl WorthQueryBoundSourceExpectation {
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
pub(in crate::domain_computation::primary_graph) use root_selection::WorthQueryObservedRootSelection;
pub(in crate::domain_computation::primary_graph) use source_identity::{
    WorthQueryCheckpointSourceIdentity, WorthQueryObservedSourceEpoch,
    WorthQueryRuntimeSourceIdentity,
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
    pub(super) parameters: std::sync::Arc<worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters>,
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
            parameters: std::sync::Arc::clone(&self.parameters),
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
        input: &Binding::Input,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let identity = source.idempotency_identity().bytes();
        self.bind_checked_source_expectation::<Binding, Scope>(admission, source, identity, input)
    }

    pub fn bind_application_result_set_expectation<Binding, Scope>(
        &self,
        admission: &mut crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        result_set: WorthQueryObservedResultSet<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let identity = result_set.idempotency_identity();
        self.bind_checked_source_expectation::<Binding, Scope>(
            admission,
            result_set.source,
            identity,
            input,
        )
    }

    fn bind_checked_source_expectation<Binding, Scope>(
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
        identity: [u8; 32],
        input: &Binding::Input,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial>
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
        let parameters_denial = || {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::SourceParametersMismatch,
                Binding::IDENTITY,
            )
        };
        if let Some(expected) =
            Binding::expected_source_parameters(input).map_err(|_| parameters_denial())?
        {
            if !source
                .parameters
                .matches_expected(expected)
                .map_err(|denial| {
                    use worth_query_admission::facade::application_query::WorthQueryApplicationQueryParameterDenialKind as Kind;
                    WorthQuerySourceExpectationDenial::new(
                        match denial.kind() {
                            Kind::CanonicalEntryBudgetExceeded | Kind::CanonicalEncodedByteBudgetExceeded => WorthQuerySourceExpectationDenialKind::WorkBudgetExceeded,
                            _ => WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                        },
                        Binding::IDENTITY,
                    )
                })?
            {
                return Err(parameters_denial());
            }
        }
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
        Ok(WorthQueryBoundSourceExpectation {
            identity,
            partition_identity,
        })
    }
}
