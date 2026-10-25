//! Borrowed law evidence from the original native refusal, never a second verdict.
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::ContractId;
use worth_relational::facade::transactions::{
    ConflictClass, EntityReference, InvariantViolationFields, TransactionCommitError,
};

/// The relation law and typed identities that refused a proposed commit.
///
/// These values identify the original violation. They grant no graph access or
/// mutation authority. A created endpoint names an entity in the refused
/// proposal, not a published entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorthQueryRelationLawDenial<'a> {
    /// This source-to-target edge closes a directed cycle in the proposed graph.
    Acyclicity {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
    },
}

impl<'a> WorthQueryRelationLawDenial<'a> {
    pub(crate) fn from_fields(fields: &'a InvariantViolationFields) -> Option<Self> {
        match fields {
            InvariantViolationFields::Acyclicity {
                contract_id,
                relation_kind_id,
                source,
                target,
            } => Some(Self::Acyclicity {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
            }),
            _ => None,
        }
    }

    pub(crate) fn from_commit_error(error: &'a TransactionCommitError) -> Option<Self> {
        let TransactionCommitError::Conflict { error, .. } = error else {
            return None;
        };
        let ConflictClass::InvariantViolation { fields, .. } = &error.class else {
            return None;
        };
        Self::from_fields(fields)
    }
}
