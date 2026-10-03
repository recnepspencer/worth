use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

mod preparation_capacity;
mod source_fact_locator;
use source_fact_locator::SourceFactLocator;

pub(super) struct PreparedSourceFactMaterialization<'source, Query> {
    source: &'source WorthQueryObservedSource<Query>,
    fields: Vec<ValidatedSourceField<'source>>,
    fact_count: usize,
}

struct ValidatedSourceField<'source>(&'source WorthQueryObservedFieldRevision);

impl<Query> WorthQueryObservedSource<Query> {
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
        self.validated_facts(layout, expected_query_identifier)
    }

    pub(in crate::domain_computation::primary_graph) fn retained_checkpoint_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<std::sync::Arc<[Fact]>, WorthQuerySourceExpectationDenial> {
        Ok(self.validated_facts(layout, &self.query_identifier)?.into())
    }

    pub(super) fn validated_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        expected_query_identifier: &str,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        self.validate_completeness(expected_query_identifier)?;
        let footprint = self.source_meaning.footprint();
        let fields = footprint
            .aspects
            .iter()
            .chain(
                footprint
                    .root_selection
                    .iter()
                    .flat_map(|selection| &selection.aspects),
            )
            .map(|aspect| {
                let contract = layout
                    .aspect_contract(&aspect.entity_name, &aspect.aspect)
                    .ok_or_else(|| source_contract_denial(aspect))?;
                ValidatedSourceField::resolve(aspect, contract)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fact_count = source_fact_count(self, expected_query_identifier)?;
        PreparedSourceFactMaterialization {
            source: self,
            fields,
            fact_count,
        }
        .into_facts()
    }
}

fn source_fact_count<Query>(
    source: &WorthQueryObservedSource<Query>,
    subject: &str,
) -> Result<usize, WorthQuerySourceExpectationDenial> {
    let footprint = source.source_meaning.footprint();
    let selected_count = footprint
        .root_selection
        .as_ref()
        .map(|selection| {
            selection
                .entities
                .len()
                .checked_add(selection.aspects.len())
                .and_then(|n| n.checked_add(selection.adjacencies.len()))
        })
        .unwrap_or(Some(0));
    footprint
        .entities
        .len()
        .checked_add(footprint.aspects.len())
        .and_then(|n| n.checked_add(footprint.adjacencies.len()))
        .and_then(|n| n.checked_add(selected_count?))
        .ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                subject,
            )
        })
}

impl<Query> PreparedSourceFactMaterialization<'_, Query> {
    pub(super) fn into_facts(self) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        let source = self.source;
        let WorthQueryObservedSourceFootprint {
            entities,
            aspects,
            adjacencies,
            root_selection,
            ..
        } = source.source_meaning.footprint();
        let selection = root_selection.as_deref();
        let mut facts = Vec::with_capacity(self.fact_count);
        facts.extend(
            entities
                .iter()
                .copied()
                .map(|entity_id| Fact::SourceEntity { entity_id }),
        );
        for field in &self.fields[..aspects.len()] {
            field.append(&mut facts);
        }
        facts.extend(adjacencies.iter().cloned().map(adjacency_fact));
        if let Some(selection) = selection {
            facts.extend(
                selection
                    .entities
                    .iter()
                    .copied()
                    .map(|entity_id| Fact::SourceEntity { entity_id }),
            );
            for field in &self.fields[aspects.len()..] {
                field.append(&mut facts);
            }
            facts.extend(selection.adjacencies.iter().cloned().map(adjacency_fact));
        }
        let mut seen = std::collections::BTreeMap::new();
        let mut unique = Vec::with_capacity(facts.len());
        for fact in facts {
            let identity = SourceFactLocator::from_fact(&fact);
            match seen.entry(identity) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(unique.len());
                    unique.push(fact);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    let existing = &mut unique[*entry.get()];
                    if !merge_same_source_fact(existing, fact) {
                        return Err(WorthQuerySourceExpectationDenial::new(
                            WorthQuerySourceExpectationDenialKind::SourceChanged,
                            source.query_identifier.as_str(),
                        ));
                    }
                }
            }
        }
        Ok(unique)
    }
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

fn merge_same_source_fact(existing: &mut Fact, duplicate: Fact) -> bool {
    match (existing, duplicate) {
        (
            Fact::SourceAdjacencyRevision {
                native_revision: first_revision,
                comparison_work_limit: first_limit,
                endpoints: first_endpoints,
                ..
            },
            Fact::SourceAdjacencyRevision {
                native_revision: second_revision,
                comparison_work_limit: second_limit,
                endpoints: second_endpoints,
                ..
            },
        ) if *first_revision == second_revision => {
            *first_limit = (*first_limit).max(second_limit);
            first_endpoints.extend(second_endpoints);
            first_endpoints.sort();
            first_endpoints.dedup();
            true
        }
        (existing, duplicate) => *existing == duplicate,
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

fn adjacency_fact(adjacency: WorthQueryObservedAdjacencyRevision) -> Fact {
    Fact::SourceAdjacencyRevision {
        relation_kind: adjacency.relation_kind,
        anchor: adjacency.anchor,
        direction: adjacency.direction,
        native_revision: adjacency.native_revision,
        comparison_work_limit: adjacency.comparison_work_limit,
        endpoints: adjacency.endpoints,
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::{
        identity::{EntityId, KindId, PartitionId, VersionId},
        runtime::RelationalAdjacencyDirection,
    };

    use super::{merge_same_source_fact, Fact};

    #[test]
    fn overlapping_path_and_projection_adjacency_merge_at_one_native_revision() {
        let anchor = EntityId::new(PartitionId::main(), 1, 1);
        let endpoint = EntityId::new(PartitionId::main(), 2, 1);
        let mut projected = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(4)),
            comparison_work_limit: 1,
            endpoints: vec![endpoint],
        };
        let path = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(4)),
            comparison_work_limit: 1,
            endpoints: vec![],
        };
        assert!(merge_same_source_fact(&mut projected, path));
        assert!(
            matches!(&projected, Fact::SourceAdjacencyRevision { endpoints, .. } if endpoints == &vec![endpoint])
        );
        let changed = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(5)),
            comparison_work_limit: 1,
            endpoints: vec![],
        };
        assert!(!merge_same_source_fact(&mut projected, changed));
    }
}
