//! Nullable existing-reference replacement, distinct from allocation and ownership transfer.
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
}
impl ApplicationOperationBodyReferenceInput {
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
        let mut record = String::from("nullable-reference-input-v1;");
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
        ] {
            write!(record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}
