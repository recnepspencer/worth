//! A certified source reuses native typed query paths, not a second path vocabulary.
mod authoring;
use crate::{
    application_query::{
        ApplicationQueryRootPathDirection as Direction, ApplicationQueryRootPathMeaning,
    },
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
        part(self.path.start_entity());
        part(self.path.terminal_entity());
        part(&self.path.steps().len().to_string());
        for step in self.path.steps() {
            part(step.relation());
            part(step.from());
            part(step.to());
            part(match step.direction() {
                Direction::Forward => "forward",
                Direction::Reverse => "reverse",
            });
        }
        part(&self.path.guards().len().to_string());
        for guard in self.path.guards() {
            part(&guard.after_step().to_string());
            part(guard.entity());
            part(guard.aspect());
            part(guard.field());
            part(guard.scalar_family().canonical_name());
            part(guard.value_type());
            part(
                worth_foundational::facade::prepare_aspect_value_identity_basis(guard.expected())
                    .as_str(),
            );
        }
        part(self.field.entity());
        part(self.field.aspect());
        part(self.field.field());
        part(self.presence.canonical_name());
        result
    }
    /// This profile selects one distinct live identity at every hop. Guarded paths
    /// are not silently weakened into unguarded captures.
    pub fn has_supported_shape(&self) -> bool {
        if !self.path.guards().is_empty() || self.path.terminal_entity() != self.field.entity() {
            return false;
        }
        let mut current = self.path.start_entity();
        for step in self.path.steps() {
            if current != step.parent_entity() {
                return false;
            }
            current = step.child_entity();
        }
        current == self.path.terminal_entity()
    }
    pub fn decision_reads(&self) -> Vec<Read> {
        let mut reads = std::collections::BTreeSet::new();
        reads.insert(Read::Entity {
            entity: self.path.start_entity().into(),
        });
        for step in self.path.steps() {
            reads.insert(Read::Entity {
                entity: step.child_entity().into(),
            });
            reads.insert(Read::Relation {
                relation: step.relation().into(),
                from: step.from().into(),
                to: step.to().into(),
            });
        }
        reads.insert(Read::Field {
            entity: self.field.entity().into(),
            aspect: self.field.aspect().into(),
            field: self.field.field().into(),
        });
        reads.into_iter().collect()
    }
}
