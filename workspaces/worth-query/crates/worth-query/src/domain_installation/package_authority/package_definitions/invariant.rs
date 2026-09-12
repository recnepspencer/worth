use std::num::NonZeroU64;

use worth_relational::facade::identity::KindId;

use super::super::{WorthQueryDomainIdentityName, WorthQueryDomainSemanticVersion};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainInvariantPredicate {
    RequiresOutgoingRelations {
        relevant_entity_kinds: Vec<KindId>,
        required_relation_kinds: Vec<KindId>,
        traversal_depth: u16,
    },
}

impl WorthQueryDomainInvariantPredicate {
    pub fn requires_outgoing_relations(
        mut relevant_entity_kinds: Vec<KindId>,
        mut required_relation_kinds: Vec<KindId>,
        traversal_depth: u16,
    ) -> Self {
        relevant_entity_kinds.sort();
        relevant_entity_kinds.dedup();
        required_relation_kinds.sort();
        required_relation_kinds.dedup();
        Self::RequiresOutgoingRelations {
            relevant_entity_kinds,
            required_relation_kinds,
            traversal_depth,
        }
    }

    pub(crate) fn canonical_part(&self) -> String {
        match self {
            Self::RequiresOutgoingRelations {
                relevant_entity_kinds,
                required_relation_kinds,
                traversal_depth,
            } => format!(
                "requires-outgoing:{}:{}:{}",
                relevant_entity_kinds
                    .iter()
                    .map(|kind| kind.as_u32().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                required_relation_kinds
                    .iter()
                    .map(|kind| kind.as_u32().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                traversal_depth
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainInvariantDefinition {
    name: WorthQueryDomainIdentityName,
    semantic_version: WorthQueryDomainSemanticVersion,
    predicate: WorthQueryDomainInvariantPredicate,
    maximum_work_units: NonZeroU64,
}

impl WorthQueryDomainInvariantDefinition {
    pub fn new(
        name: WorthQueryDomainIdentityName,
        semantic_version: WorthQueryDomainSemanticVersion,
        predicate: WorthQueryDomainInvariantPredicate,
        maximum_work_units: NonZeroU64,
    ) -> Self {
        Self {
            name,
            semantic_version,
            predicate,
            maximum_work_units,
        }
    }

    pub fn name(&self) -> &WorthQueryDomainIdentityName {
        &self.name
    }

    pub fn semantic_version(&self) -> WorthQueryDomainSemanticVersion {
        self.semantic_version
    }

    pub fn maximum_work_units(&self) -> NonZeroU64 {
        self.maximum_work_units
    }

    pub fn predicate(&self) -> &WorthQueryDomainInvariantPredicate {
        &self.predicate
    }

    pub(crate) fn slot_key(&self) -> String {
        format!("{}:{}", self.name.as_str(), self.semantic_version.major())
    }

    pub(crate) fn canonical_part(&self) -> String {
        format!(
            "{}:{}.{}:{}:work={}",
            self.name.as_str(),
            self.semantic_version.major(),
            self.semantic_version.minor(),
            self.predicate.canonical_part(),
            self.maximum_work_units
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_budget_changes_invariant_canonical_meaning() {
        let definition = |work| {
            WorthQueryDomainInvariantDefinition::new(
                WorthQueryDomainIdentityName::new("requires-links").unwrap(),
                WorthQueryDomainSemanticVersion::new(1, 0),
                WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
                    vec![KindId::new(1)],
                    vec![KindId::new(2)],
                    1,
                ),
                NonZeroU64::new(work).unwrap(),
            )
        };
        assert_ne!(
            definition(4096).canonical_part(),
            definition(4097).canonical_part()
        );
        assert_eq!(definition(4096).maximum_work_units().get(), 4096);
    }
}
