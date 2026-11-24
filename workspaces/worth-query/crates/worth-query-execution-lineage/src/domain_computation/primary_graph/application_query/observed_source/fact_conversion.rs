use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

pub mod control;
pub mod field_preparation;
pub mod materialization;
pub use control::SourceFactExpansionControl;
pub mod retention_validation;
pub mod source_fact_locator;
use source_fact_locator::SourceFactLocator;

/// Complete owner-computed work, retained payload, and temporary peak scratch
/// quote for converting one [`WorthQueryObservedSource`] via `validated_facts`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourceFactConversionQuote {
    pub fact_count: usize,
    pub field_count: usize,
    pub adjacency_count: usize,
    pub adjacency_endpoint_count: usize,
    /// Surviving heap bytes owned by materialized `Fact` payloads (such as
    /// `CanonicalFieldPath` vectors and cloned `AspectKey`/`FieldKey` strings)
    /// that outlive temporary conversion scratch.
    pub retained_payload_bytes: u64,
    /// Peak temporary scratch bytes live inside `validated_facts` concurrently
    /// with any outer retained buffer (validation BTreeMaps OR prepared fields,
    /// both materialized `Vec<Fact>` buffers, `merge_facts` BTreeMap, and cloned
    /// `SourceFactLocator` path/string payloads).
    pub peak_scratch_bytes: u64,
    /// Bounded O(N log N) conversion-owned validation, preparation, and merge
    /// work units. Adjacency endpoint copy/sort/merge work is debited directly
    /// once by `AdmittedAdjacencyEndpoints`.
    pub conversion_work: u64,
}

fn n_log2_n(count: usize) -> Option<u64> {
    if count <= 1 {
        return u64::try_from(count).ok();
    }
    let log2 = u64::from(usize::BITS - count.leading_zeros()).max(1);
    u64::try_from(count).ok()?.checked_mul(log2)
}

pub struct PreparedSourceFactMaterialization<'source, 'request, 'authority> {
    footprint: &'source WorthQueryObservedSourceFootprint,
    subject: &'source str,
    managed: Option<
        &'source std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
        >,
    >,
    authorization: Option<
        &'source std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization,
        >,
    >,
    fields: Vec<ValidatedSourceField<'source>>,
    fact_count: usize,
    control: SourceFactExpansionControl<'request, 'authority>,
}

struct ValidatedSourceField<'source>(&'source WorthQueryObservedFieldRevision);

impl<Query> WorthQueryObservedSource<Query> {
    pub(crate) fn require_supported_fact_expansion(
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
    pub fn validate_and_into_facts(
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

    pub(crate) fn retained_checkpoint_facts(
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

    pub fn validated_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        control: SourceFactExpansionControl<'_, '_>,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        self.admit_fact_materialization(layout, subject, control)?
            .into_facts()
    }

    /// Computes exact owner-side retained payload bytes, peak temporary scratch
    /// bytes, and O(N log N) work for `validated_facts` before allocation.
    pub fn fact_conversion_quote(&self) -> Option<SourceFactConversionQuote> {
        use worth_foundational::facade::FieldKey;
        use worth_query_execution_foundation::facade::primary_graph::admitted_capture::storage::tree_clone_bytes;
        use worth_relational::facade::identity::{EntityId, KindId};

        let fp = self.source_meaning.footprint();
        let fact_count = fp.fact_count()?;
        let field_count = fp.field_count()?;
        let mut adjacency_count = 0_usize;
        let mut adjacency_endpoint_count = 0_usize;
        let mut retained_payload_bytes = 0_u64;
        for part in fp.fact_sources() {
            adjacency_count = adjacency_count.checked_add(part.adjacencies.len())?;
            for adjacency in part.adjacencies {
                adjacency_endpoint_count =
                    adjacency_endpoint_count.checked_add(adjacency.endpoints.len())?;
            }
            for aspect in part.aspects {
                let per_field = (std::mem::size_of::<FieldKey>() as u64)
                    .checked_add(u64::try_from(aspect.aspect.as_str().len()).ok()?)?
                    .checked_add(u64::try_from(aspect.field.as_str().len()).ok()?)?;
                retained_payload_bytes = retained_payload_bytes.checked_add(per_field)?;
            }
        }
        let validation_peak = tree_clone_bytes::<(EntityId, usize, usize, u64)>(field_count)?
            .checked_add(tree_clone_bytes::<(EntityId, KindId, u8, u64)>(
                adjacency_count,
            )?)?;
        let prepared_fields_bytes = u64::try_from(
            field_count.checked_mul(std::mem::size_of::<ValidatedSourceField<'_>>())?,
        )
        .ok()?;
        let two_fact_vecs = u64::try_from(
            fact_count
                .checked_mul(2)?
                .checked_mul(std::mem::size_of::<Fact>())?,
        )
        .ok()?;
        let merge_tree = tree_clone_bytes::<(SourceFactLocator, usize)>(fact_count)?;
        // `merge_facts` clones every field locator (aspect string + single-element
        // CanonicalFieldPath vec + field string) into `seen` while both Fact Vecs live.
        let merge_peak = prepared_fields_bytes
            .checked_add(two_fact_vecs)?
            .checked_add(merge_tree)?
            .checked_add(retained_payload_bytes)?;
        let peak_scratch_bytes = validation_peak.max(merge_peak);
        let conversion_work = n_log2_n(field_count)?
            .checked_add(n_log2_n(adjacency_count)?)?
            .checked_add(n_log2_n(fact_count)?)?
            .checked_add(u64::try_from(field_count).ok()?)?
            .checked_add(u64::try_from(fact_count).ok()?)?
            .checked_add(1)?;
        Some(SourceFactConversionQuote {
            fact_count,
            field_count,
            adjacency_count,
            adjacency_endpoint_count,
            retained_payload_bytes,
            peak_scratch_bytes,
            conversion_work,
        })
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
    pub fn validate_affinity(
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

mod court_doors_for_test;

pub fn applied_facts(
    footprint: &WorthQueryObservedSourceFootprint,
    layout: &WorthQueryPrimaryGraphLayout,
    subject: &str,
    control: SourceFactExpansionControl<'_, '_>,
) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
    control.poll(subject)?;
    if !footprint.complete {
        return Err(WorthQuerySourceExpectationDenial::new(
            WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
            subject,
        ));
    }
    retention_validation::validate_footprint(footprint, layout, subject, control)?;
    field_preparation::prepare_fields(footprint, subject, None, None, control)?.into_facts()
}
