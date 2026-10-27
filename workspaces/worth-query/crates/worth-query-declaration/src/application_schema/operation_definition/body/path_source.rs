//! A certified source reuses native typed query paths, not a second path vocabulary.
mod authoring;
use crate::{
    application_query::ApplicationQueryRootPathMeaning,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldPresence,
        ApplicationOperationDecisionReadTarget as Read,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyPathFieldSource {
    pub path: ApplicationQueryRootPathMeaning,
    pub field: ApplicationFieldBindingLocus,
    pub presence: ApplicationFieldPresence,
}
impl ApplicationOperationBodyPathFieldSource {
    pub(super) fn has_closed_reads(
        &self,
        operation: &str,
        members: &[crate::application_schema::ApplicationSchemaMember],
    ) -> bool {
        use crate::application_schema::ApplicationSchemaMember as Member;
        self.has_supported_shape()
            && self.path.steps().iter().all(|step| members.iter().any(|member| matches!(member,
                Member::Relation { relation, from, to, .. } if relation == step.relation() && from == step.from() && to == step.to())))
            && self.decision_reads().iter().all(|read| members.iter().any(|member| matches!(member,
                Member::OperationDecisionRead { operation: declared, target } if declared == operation && target == read)))
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut part =
            |text: &str| write!(&mut result, "{}:{text};", text.len()).expect("String write");
        part("body-path-field-source-v1");
        self.path.visit_canonical_parts(&mut part);
        part(self.field.entity());
        part(self.field.aspect());
        part(self.field.field());
        part(self.presence.canonical_name());
        result
    }
    /// This profile selects one distinct live identity at every hop. Guarded paths
    /// are not silently weakened into unguarded captures.
    pub fn has_supported_shape(&self) -> bool {
        supported_path(&self.path) && self.path.terminal_entity() == self.field.entity()
    }
    pub fn decision_reads(&self) -> Vec<Read> {
        let mut reads = path_reads(&self.path)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        reads.insert(Read::Field {
            entity: self.field.entity().into(),
            aspect: self.field.aspect().into(),
            field: self.field.field().into(),
        });
        reads.into_iter().collect()
    }
}

pub(super) fn supported_path(path: &ApplicationQueryRootPathMeaning) -> bool {
    if !path.guards().is_empty() {
        return false;
    }
    let mut current = path.start_entity();
    for step in path.steps() {
        if step.traversal() != crate::application_query::ApplicationQueryRootPathTraversal::OneHop
            || current != step.parent_entity()
        {
            return false;
        }
        current = step.child_entity();
    }
    current == path.terminal_entity()
}
pub(super) fn path_reads(path: &ApplicationQueryRootPathMeaning) -> Vec<Read> {
    let mut reads = std::collections::BTreeSet::from([Read::Entity {
        entity: path.start_entity().into(),
    }]);
    for step in path.steps() {
        reads.insert(Read::Entity {
            entity: step.child_entity().into(),
        });
        reads.insert(Read::Relation {
            relation: step.relation().into(),
            from: step.from().into(),
            to: step.to().into(),
        });
    }
    reads.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application_query::{
        ApplicationQueryRootPathStep, ApplicationQueryRootPathTraversal as Mode, RelationDirection,
        WorthQueryPortableApplicationQueryRootPathParts,
    };

    #[test]
    fn path_field_source_canonical_record_preserves_existing_forward_reverse_bytes() {
        let steps = [RelationDirection::Forward, RelationDirection::Reverse]
            .into_iter()
            .map(|direction| {
                ApplicationQueryRootPathStep::from_untrusted_fields(
                    "Children".into(),
                    "Node".into(),
                    "Child".into(),
                    direction,
                    Mode::OneHop,
                )
            })
            .collect();
        let source = ApplicationOperationBodyPathFieldSource {
            path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationQueryRootPathParts {
                    start_entity: "Node".into(),
                    terminal_entity: "Node".into(),
                    steps,
                    guards: Vec::new(),
                },
            ),
            field: ApplicationFieldBindingLocus::new("Node", "Facts", "Text"),
            presence: ApplicationFieldPresence::Required,
        };
        assert!(source.has_supported_shape());
        assert_eq!(
            source.canonical_record(),
            concat!(
                "25:body-path-field-source-v1;4:Node;4:Node;1:2;",
                "8:Children;4:Node;5:Child;7:forward;",
                "8:Children;4:Node;5:Child;7:reverse;",
                "1:0;4:Node;5:Facts;4:Text;8:required;",
            )
        );
    }
    #[test]
    fn operation_path_profile_refuses_both_closure_directions_without_weakening_one_hop() {
        for direction in [RelationDirection::Forward, RelationDirection::Reverse] {
            for mode in [Mode::OneHop, Mode::Closure] {
                let path = ApplicationQueryRootPathMeaning::from_untrusted_parts(
                    WorthQueryPortableApplicationQueryRootPathParts {
                        start_entity: "Node".into(),
                        terminal_entity: "Node".into(),
                        guards: vec![],
                        steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                            "Next".into(),
                            "Node".into(),
                            "Node".into(),
                            direction,
                            mode,
                        )],
                    },
                );
                assert_eq!(supported_path(&path), mode == Mode::OneHop);
            }
        }
    }
}
