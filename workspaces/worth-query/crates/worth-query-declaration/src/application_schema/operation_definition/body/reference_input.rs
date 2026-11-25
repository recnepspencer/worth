//! Existing-reference replacement or exact-edge toggling, distinct from allocation and
//! ownership transfer.
mod authoring;
mod validation;
use crate::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget as Read,
        ApplicationOperationProgramTarget as Target,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
pub(super) use validation::{validate, validate_shape};
use worth_foundational::facade::ScalarAspectType;
/// How the captured input selects the scope's reference edge.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyReferenceInputMode {
    /// `Option<key>` replaces an at-most-one outgoing selection; explicit `None` clears it.
    ReplaceNullable,
    /// A required key and the bool `linked_member`: link the exact edge when true and
    /// absent, unlink it when false and present, and otherwise change nothing.
    ToggleExact { linked_member: String },
}
/// Inert correspondence. A captured business key grants no graph or target authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyReferenceInput {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub field: ApplicationFieldBindingLocus,
    pub binding: WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
    pub member: String,
    pub key: String,
    pub mode: ApplicationOperationBodyReferenceInputMode,
}
impl ApplicationOperationBodyReferenceInput {
    /// Replacement captures `Option<key>`; a toggle requires its exact target key.
    pub fn target_is_optional(&self) -> bool {
        self.mode == ApplicationOperationBodyReferenceInputMode::ReplaceNullable
    }
    /// Every name, including a toggle's bool member, is non-empty and unpadded.
    pub(in crate::application_schema::operation_definition::body) fn has_trimmed_names(
        &self,
    ) -> bool {
        [
            self.relation.as_str(),
            &self.from,
            &self.to,
            &self.member,
            &self.key,
        ]
        .into_iter()
        .chain(self.linked_member())
        .all(|part| !part.is_empty() && part.trim() == part)
    }
    pub fn linked_member(&self) -> Option<&str> {
        match &self.mode {
            ApplicationOperationBodyReferenceInputMode::ReplaceNullable => None,
            ApplicationOperationBodyReferenceInputMode::ToggleExact { linked_member } => {
                Some(linked_member)
            }
        }
    }
    pub fn decision_reads(&self) -> [Read; 4] {
        [
            Read::Entity {
                entity: self.from.clone(),
            },
            Read::Entity {
                entity: self.to.clone(),
            },
            Read::Relation {
                relation: self.relation.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
            },
            Read::Field {
                entity: self.field.entity().into(),
                aspect: self.field.aspect().into(),
                field: self.field.field().into(),
            },
        ]
    }
    pub fn program_targets(&self) -> [Target; 2] {
        [
            Target::Link {
                relation: self.relation.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
            },
            Target::Unlink {
                relation: self.relation.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
            },
        ]
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from(match self.mode {
            ApplicationOperationBodyReferenceInputMode::ReplaceNullable => {
                "nullable-reference-input-v1;"
            }
            ApplicationOperationBodyReferenceInputMode::ToggleExact { .. } => {
                "exact-reference-toggle-v1;"
            }
        });
        for part in [
            &self.relation,
            &self.from,
            &self.to,
            self.field.entity(),
            self.field.aspect(),
            self.field.field(),
            self.binding.as_str(),
            self.scalar_family.canonical_name(),
            &self.member,
            &self.key,
        ]
        .into_iter()
        .chain(self.linked_member())
        {
            write!(record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}
