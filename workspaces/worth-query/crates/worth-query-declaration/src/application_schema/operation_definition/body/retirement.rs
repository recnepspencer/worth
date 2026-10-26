//! A source-input change retires current selections, never their owning rows.
mod authoring;
mod validation;
use super::{ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField as Step};
use crate::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget as Read,
        ApplicationOperationProgramTarget as Target,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use std::num::NonZeroUsize;
pub(super) use validation::validate;
use worth_foundational::facade::ScalarAspectType;

/// Authority-free description; installation binds exact fields, ownership and effects.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyOwnedSelectionRetirement {
    pub trigger: ApplicationFieldBindingLocus,
    pub binding: WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
    pub member: String,
    pub owners_relation: String,
    pub scope: String,
    pub owner: String,
    pub selection_relation: String,
    pub selected: String,
    pub maximum_owners: NonZeroUsize,
}
impl ApplicationOperationBodyOwnedSelectionRetirement {
    /// Finds the original input slot of the exact source write, not a second input getter.
    pub fn source_input_index(&self, body: &ApplicationOperationBodyProgram) -> Option<usize> {
        body.scope_fields()
            .iter()
            .filter(|step| step.input_member().is_some())
            .position(|step| {
                matches!(
                    step,
                    Step::AssignInput { .. } | Step::SetOptionalInput { .. }
                ) && step.field() == &self.trigger
                    && step.binding() == &self.binding
                    && step.scalar_family() == self.scalar_family
                    && step.input_member() == Some(self.member.as_str())
            })
    }
    /// Descriptive profile compatibility; this grants no admission or mutation authority.
    pub fn fits_body(&self, body: &ApplicationOperationBodyProgram) -> bool {
        body.scope_entity() == Some(self.scope.as_str())
            && self.selected != self.scope
            && self.selected != self.owner
            && body
                .retirements()
                .iter()
                .all(|other| self.selected != other.owner)
            && self.trigger.entity() == self.scope
            && !self.member.is_empty()
            && self.member.trim() == self.member
            && self.source_input_index(body).is_some()
            && body
                .creation()
                .is_none_or(|creation| creation.signing().is_none())
    }
    pub fn decision_reads(&self) -> Vec<Read> {
        vec![
            Read::Entity {
                entity: self.scope.clone(),
            },
            Read::Entity {
                entity: self.owner.clone(),
            },
            Read::Entity {
                entity: self.selected.clone(),
            },
            Read::Field {
                entity: self.trigger.entity().into(),
                aspect: self.trigger.aspect().into(),
                field: self.trigger.field().into(),
            },
            Read::Relation {
                relation: self.owners_relation.clone(),
                from: self.scope.clone(),
                to: self.owner.clone(),
            },
            Read::Relation {
                relation: self.selection_relation.clone(),
                from: self.owner.clone(),
                to: self.selected.clone(),
            },
        ]
    }
    pub fn program_targets(&self) -> [Target; 2] {
        [
            Target::Unlink {
                relation: self.selection_relation.clone(),
                from: self.owner.clone(),
                to: self.selected.clone(),
            },
            Target::Delete {
                entity: self.selected.clone(),
            },
        ]
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("owned-selection-retirement-v1;");
        for part in [
            self.trigger.entity(),
            self.trigger.aspect(),
            self.trigger.field(),
            self.binding.as_str(),
            self.scalar_family.canonical_name(),
            &self.member,
            &self.owners_relation,
            &self.scope,
            &self.owner,
            &self.selection_relation,
            &self.selected,
            &self.maximum_owners.get().to_string(),
        ] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}
