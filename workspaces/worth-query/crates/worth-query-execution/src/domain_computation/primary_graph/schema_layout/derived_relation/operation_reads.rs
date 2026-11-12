//! Reverse derived-read membership shares one operation set per common scope.
use super::Member;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type OperationSet = Arc<BTreeSet<Arc<str>>>;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct DerivedReadOperations {
    inline: Arc<[Arc<str>]>,
    shared: Vec<OperationSet>,
}
impl DerivedReadOperations {
    pub(super) fn for_relation(
        members: &[Member],
        relation: &str,
        shared: Vec<OperationSet>,
    ) -> Self {
        let inline = members
            .iter()
            .filter_map(|member| match member {
                Member::OperationDerivedRead {
                    operation,
                    relation: candidate,
                } if candidate == relation => Some(Arc::from(operation.as_str())),
                _ => None,
            })
            .collect();
        Self { inline, shared }
    }
    pub(in crate::domain_computation::primary_graph) fn iter(
        &self,
    ) -> impl Iterator<Item = &Arc<str>> {
        self.inline
            .iter()
            .chain(self.shared.iter().flat_map(|operations| operations.iter()))
    }
}

pub(super) fn shared_index(members: &[Member]) -> BTreeMap<String, Vec<OperationSet>> {
    let mut references = BTreeMap::<&str, BTreeSet<Arc<str>>>::new();
    for member in members {
        if let Member::OperationDecisionReadScope { operation, scope } = member {
            references
                .entry(scope)
                .or_default()
                .insert(Arc::from(operation.as_str()));
        }
    }
    let mut relations = BTreeMap::<String, Vec<OperationSet>>::new();
    for member in members {
        let Member::DecisionReadScope { definition } = member else {
            continue;
        };
        let Some(operations) = references.remove(definition.identity()) else {
            continue;
        };
        let operations = Arc::new(operations);
        for relation in definition.derived_relations() {
            relations
                .entry(relation.clone())
                .or_default()
                .push(Arc::clone(&operations));
        }
    }
    relations
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_declaration::facade::application_schema::ApplicationDecisionReadScopeMeaning;

    #[test]
    fn shared_derived_membership_keeps_exact_operations_and_one_allocation() {
        let definition = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
            Vec::new(),
            vec!["Owns".into(), "Touches".into()],
        )
        .unwrap();
        let mut members = ["Read", "Inspect"]
            .into_iter()
            .map(|operation| Member::OperationDecisionReadScope {
                operation: operation.into(),
                scope: definition.identity().into(),
            })
            .collect::<Vec<_>>();
        members.push(Member::DecisionReadScope { definition });
        members.push(Member::OperationDerivedRead {
            operation: "Inline".into(),
            relation: "Owns".into(),
        });
        let shared = shared_index(&members);
        assert!(Arc::ptr_eq(&shared["Owns"][0], &shared["Touches"][0]));
        let owns = DerivedReadOperations::for_relation(&members, "Owns", shared["Owns"].clone());
        let actual = owns
            .iter()
            .map(|name| name.as_ref())
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, BTreeSet::from(["Read", "Inspect", "Inline"]));
        assert!(!shared.contains_key("Outside"));
        let outside = DerivedReadOperations::for_relation(&members, "Outside", Vec::new());
        assert!(outside.iter().next().is_none());
    }
}
