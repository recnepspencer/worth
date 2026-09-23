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

mod fact_conversion;
mod footprint_accounting;
mod result_set;
mod root_selection;
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
pub(super) mod source_identity;
pub(in crate::domain_computation::primary_graph) use root_selection::WorthQueryObservedRootSelection;
pub(in crate::domain_computation::primary_graph) use source_identity::WorthQueryObservedSourceEpoch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceExpectationDenialKind {
    MissingExpectation,
    ForeignApplication,
    ForeignInstallation,
    ForeignSchema,
    ForeignModel,
    ForeignBranch,
    SourceRetired,
    SourceChanged,
    IncompleteFootprint,
    SourceContractMismatch,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySourceExpectationDenial {
    kind: WorthQuerySourceExpectationDenialKind,
    subject: String,
}

impl WorthQuerySourceExpectationDenial {
    pub const fn kind(&self) -> WorthQuerySourceExpectationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    #[doc(hidden)]
    pub fn new_missing(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQuerySourceExpectationDenialKind::MissingExpectation,
            subject,
        )
    }

    pub(in crate::domain_computation) fn new(
        kind: WorthQuerySourceExpectationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQuerySourceExpectationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "source expectation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQuerySourceExpectationDenial {}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph) root: EntityId,
    pub(in crate::domain_computation::primary_graph) complete: bool,
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedAspectRevision>,
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
    pub(in crate::domain_computation) query_identifier: String,
    pub(in crate::domain_computation) branch: BranchId,
    pub(in crate::domain_computation) selection: WorthQueryApplicationBasisSelectionIdentity,
    pub(in crate::domain_computation) model_root: EntityId,
    pub(super) footprint: WorthQueryObservedSourceFootprint,
    pub(super) source_identity: [u8; 32],
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
            footprint: self.footprint.clone(),
            source_identity: self.source_identity,
            _marker: PhantomData,
        }
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation) fn partition_identity(&self) -> [u8; 32] {
        source_identity::derive_partition_identity(
            self.query_identity.as_bytes(),
            self.parameter_binding_identity.bytes(),
        )
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
        self.footprint.complete.then_some(()).ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
                subject,
            )
        })
    }

    /// Collision-resistant digest of the complete normalized source footprint.
    /// Runtime, installation, query, and branch affinity are validated before
    /// idempotency lookup; branch-local sibling work leaves this digest intact.
    pub(in crate::domain_computation) fn idempotency_identity(&self) -> [u8; 32] {
        self.source_identity
    }

    pub(in crate::domain_computation::primary_graph) fn output_source_epoch(
        &self,
    ) -> Option<source_identity::WorthQueryObservedSourceEpoch> {
        source_identity::WorthQueryObservedSourceEpoch::from_observation(
            self.query_identity.as_bytes(),
            self.parameter_binding_identity.bytes(),
            &self.footprint,
            &self.selection,
            self.source_identity,
        )
    }

    pub(in crate::domain_computation::primary_graph) const fn source_root(&self) -> EntityId {
        self.footprint.root
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn footprint_for_test(
        &self,
    ) -> &WorthQueryObservedSourceFootprint {
        &self.footprint
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
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let identity = source.idempotency_identity();
        self.bind_checked_source_expectation::<Binding, Scope>(admission, source, identity)
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
