//! Record expansion deduplicates immutable access loci, not live codec carriers.
use super::{Member, ReadTarget, WriteTarget};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum AccessKind {
    ReadField,
    WriteField,
    RemoveAspect,
}

#[cfg_attr(not(test), derive(Eq, PartialEq, Ord, PartialOrd))]
pub(super) struct MemberKey {
    kind: AccessKind,
    operation: String,
    entity: String,
    aspect: String,
    field: String,
}

#[cfg(test)]
impl MemberKey {
    fn comparison_fields(&self) -> (&AccessKind, &str, &str, &str, &str) {
        (
            &self.kind,
            &self.operation,
            &self.entity,
            &self.aspect,
            &self.field,
        )
    }
}
#[cfg(test)]
impl Ord for MemberKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        super::comparison_observation::comparison();
        self.comparison_fields().cmp(&other.comparison_fields())
    }
}
#[cfg(test)]
impl PartialOrd for MemberKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[cfg(test)]
impl PartialEq for MemberKey {
    fn eq(&self, other: &Self) -> bool {
        super::comparison_observation::comparison();
        self.comparison_fields() == other.comparison_fields()
    }
}
#[cfg(test)]
impl Eq for MemberKey {}

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
