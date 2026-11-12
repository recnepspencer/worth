//! Record expansion deduplicates immutable access loci, not live codec carriers.
use super::{Member, ReadTarget, WriteTarget};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum AccessKind {
    ReadField,
    WriteField,
    RemoveAspect,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct MemberKey {
    kind: AccessKind,
    operation: String,
    entity: String,
    aspect: String,
    field: String,
}

pub(super) fn member_key(member: &Member) -> Option<MemberKey> {
    let (kind, operation, entity, aspect, field) = match member {
        Member::OperationDecisionRead {
            operation,
            target:
                ReadTarget::Field {
                    entity,
                    aspect,
                    field,
                },
        } => (
            AccessKind::ReadField,
            operation,
            entity,
            aspect,
            field.as_str(),
        ),
        Member::OperationProgram {
            operation,
            target:
                WriteTarget::Write {
                    entity,
                    aspect,
                    field,
                },
        } => (
            AccessKind::WriteField,
            operation,
            entity,
            aspect,
            field.as_str(),
        ),
        Member::OperationProgram {
            operation,
            target: WriteTarget::RemoveAspect { entity, aspect },
        } => (AccessKind::RemoveAspect, operation, entity, aspect, ""),
        _ => return None,
    };
    Some(MemberKey {
        kind,
        operation: operation.clone(),
        entity: entity.clone(),
        aspect: aspect.clone(),
        field: field.to_owned(),
    })
}
