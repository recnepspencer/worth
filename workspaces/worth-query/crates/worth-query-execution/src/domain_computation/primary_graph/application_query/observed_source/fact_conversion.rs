use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

mod control;
mod field_preparation;
mod materialization;
pub(in crate::domain_computation::primary_graph) use control::SourceFactExpansionControl;
mod retention_validation;
mod source_fact_locator;
use source_fact_locator::SourceFactLocator;

pub(in crate::domain_computation::primary_graph::application_query) struct PreparedSourceFactMaterialization<
    'source,
    'request,
    'authority,
    Query,
> {
    source: &'source WorthQueryObservedSource<Query>,
    fields: Vec<ValidatedSourceField<'source>>,
    fact_count: usize,
    control: SourceFactExpansionControl<'request, 'authority>,
}

struct ValidatedSourceField<'source>(&'source WorthQueryObservedFieldRevision);

impl<Query> WorthQueryObservedSource<Query> {
    pub(super) fn require_supported_fact_expansion(
        &self,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        if self
            .source_meaning
            .footprint()
            .root_selection
            .as_ref()
            .is_some_and(|selection| selection.managed_index.is_some())
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ManagedSourceExpansionUnsupported,
                subject,
            ));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn validate_and_into_facts(
        self,
        runtime_authority: u64,
        binding: &ApplicationSchemaBindingIdentity,
        branch: &BranchId,
        model_root: EntityId,
        selected_product: &crate::basis::WorthQueryProductBranchReadIdentity,
        expected_query_identifier: &str,
        expected_query_identity: &WorthQueryInstalledApplicationQueryIdentity,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        self.validate_affinity(
            runtime_authority,
            binding,
            branch,
            model_root,
            selected_product,
            expected_query_identifier,
            expected_query_identity,
        )?;
        self.validated_facts(
            layout,
            expected_query_identifier,
            SourceFactExpansionControl::in_request(*execution, request),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn retained_checkpoint_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<std::sync::Arc<[Fact]>, WorthQuerySourceExpectationDenial> {
        Ok(self
            .validated_facts(layout, &self.query_identifier, SourceFactExpansionControl::legacy(
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(allocation_policy, request)))?
            .into())
    }

    pub(super) fn validated_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        control: SourceFactExpansionControl<'_, '_>,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        self.admit_fact_materialization(layout, subject, control)?
            .into_facts()
    }
}

fn source_fact_count<Query>(
    source: &WorthQueryObservedSource<Query>,
    subject: &str,
) -> Result<usize, WorthQuerySourceExpectationDenial> {
    let footprint = source.source_meaning.footprint();
    footprint.fact_count().ok_or_else(|| {
        WorthQuerySourceExpectationDenial::new(
            WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
            subject,
        )
    })
}

impl<Query> WorthQueryObservedSource<Query> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn validate_affinity(
        &self,
        runtime_authority: u64,
        binding: &ApplicationSchemaBindingIdentity,
        branch: &BranchId,
        model_root: EntityId,
        selected_product: &crate::basis::WorthQueryProductBranchReadIdentity,
        expected_query_identifier: &str,
        expected_query_identity: &WorthQueryInstalledApplicationQueryIdentity,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        use WorthQuerySourceExpectationDenialKind as Kind;
        let deny = |kind| WorthQuerySourceExpectationDenial::new(kind, expected_query_identifier);
        if self.runtime_authority != runtime_authority {
            return Err(deny(Kind::ForeignApplication));
        }
        if self.schema_binding.runtime_ordinal() != binding.runtime_ordinal()
            || self.schema_binding.generation() != binding.generation()
        {
            return Err(deny(Kind::ForeignInstallation));
        }
        if self.schema_binding.schema_identity() != binding.schema_identity()
            || self.schema_binding.package_identity() != binding.package_identity()
        {
            return Err(deny(Kind::ForeignSchema));
        }
        if self.query_identifier.as_str() != expected_query_identifier
            || &self.query_identity != expected_query_identity
        {
            return Err(deny(Kind::SourceContractMismatch));
        }
        let WorthQueryApplicationBasisSelectionIdentity::Product(observed_product) =
            &self.selection
        else {
            return Err(deny(Kind::ForeignBranch));
        };
        if self.branch.as_ref() != branch
            || !selected_product.same_branch_occurrence(observed_product)
        {
            return Err(deny(Kind::ForeignBranch));
        }
        if self.model_root != model_root {
            return Err(deny(Kind::ForeignModel));
        }
        Ok(())
    }
}

fn source_contract_denial(
    aspect: &WorthQueryObservedFieldRevision,
) -> WorthQuerySourceExpectationDenial {
    WorthQuerySourceExpectationDenial::new(
        WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
        aspect.aspect.as_str(),
    )
}

impl<'source> ValidatedSourceField<'source> {
    fn resolve(
        aspect: &'source WorthQueryObservedFieldRevision,
        contract: &worth_foundational::facade::AspectContract,
    ) -> Result<Self, WorthQuerySourceExpectationDenial> {
        if contract.revision() != aspect.contract_revision {
            return Err(source_contract_denial(aspect));
        }
        let declared = match contract.shape() {
            worth_foundational::facade::AspectShape::Struct(shape) => {
                shape.field(&aspect.field).is_some()
            }
            worth_foundational::facade::AspectShape::Scalar(_) => true,
            _ => false,
        };
        if !declared {
            return Err(source_contract_denial(aspect));
        }
        Ok(Self(aspect))
    }

    fn append(&self, facts: &mut Vec<Fact>) {
        let aspect = self.0;
        facts.push(Fact::SourceFieldRevision {
            entity_id: aspect.entity,
            locator: worth_foundational::facade::AspectFieldLocator::new(
                worth_foundational::facade::LocatorAuthority::Authoritative,
                aspect.aspect.clone(),
                worth_foundational::facade::CanonicalFieldPath::single(aspect.field.clone()),
            ),
            native_revision: aspect.native_revision,
        });
    }
}

fn adjacency_fact(adjacency: &WorthQueryObservedAdjacencyRevision, control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>) -> Result<Fact, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>{
    Ok(Fact::SourceAdjacencyRevision {
        relation_kind: adjacency.relation_kind,
        anchor: adjacency.anchor,
        direction: adjacency.direction,
        native_revision: adjacency.native_revision,
        comparison_work_limit: adjacency.comparison_work_limit,
        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::preserve(&adjacency.endpoints, control)?,
    })
}

#[cfg(test)]
mod tests;
