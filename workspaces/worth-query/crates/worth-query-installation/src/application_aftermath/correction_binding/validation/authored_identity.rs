//! Resolve authored lifecycle correspondence against real operation and field meaning.
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredCorrectionBinding, PortableCorrectionMechanism},
    application_operation::{
        ApplicationMutationBindingDescriptor, ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
    },
    application_schema::{
        ApplicationFieldPresence, ApplicationOperationProgramTarget as Target,
        ApplicationSchemaMember as Member,
    },
};

pub(super) fn validate(
    members: &[Member],
    source: &ApplicationMutationBindingDescriptor,
    inverse: &ApplicationMutationBindingDescriptor,
    binding: &DeclaredCorrectionBinding,
    mechanism: Option<&PortableCorrectionMechanism>,
) -> Result<(), ()> {
    let lifecycle = members.iter().any(|member| matches!(member,
        Member::OperationProgram { operation, target: Target::Create { .. } | Target::Delete { .. } }
        if operation == source.operation_name()));
    let Some(identity) = binding.authored_identity() else {
        return if lifecycle
            && matches!(
                mechanism,
                Some(PortableCorrectionMechanism::RecordedInverse(_))
            ) {
            Err(())
        } else {
            Ok(())
        };
    };
    let Some(PortableCorrectionMechanism::RecordedInverse(recorded)) = mechanism else {
        return Err(());
    };
    let parts = identity.parts();
    let Member::Field {
        entity,
        aspect,
        field,
        presence: ApplicationFieldPresence::Required,
        unit,
        writable: false,
        equality_queryable: false,
        unique: false,
        ..
    } = parts.logical_key.as_ref()
    else {
        return Err(());
    };
    if parts.family.as_str().trim().is_empty() || parts.binding_unit != *unit {
        return Err(());
    }
    let actual = members
        .iter()
        .find(|member| {
            matches!(member, Member::Field { entity: e, aspect: a, field: f, .. }
        if e == entity && a == aspect && f == field)
        })
        .ok_or(())?;
    let mut normalized = actual.clone();
    if let Member::Field {
        writable,
        equality_queryable,
        unique,
        ..
    } = &mut normalized
    {
        *writable = false;
        *equality_queryable = false;
        *unique = false;
    }
    if &normalized != parts.logical_key.as_ref() {
        return Err(());
    }
    let inverse_posture = match parts.source_posture {
        Posture::Create => Posture::Retire,
        Posture::Retire => Posture::Create,
        Posture::Preserve => return Err(()),
    };
    require_role(source, &parts.source_role, entity, parts.source_posture)?;
    require_role(inverse, &parts.inverse_role, entity, inverse_posture)?;
    require_lifecycle(
        members,
        source.operation_name(),
        entity,
        parts.source_posture,
    )?;
    require_lifecycle(members, inverse.operation_name(), entity, inverse_posture)?;
    if parts.source_posture == Posture::Retire
        && !recorded.preimage_demand().loci().iter().any(|locus| {
            locus.entity() == entity && locus.aspect() == aspect && locus.field() == field
        })
    {
        return Err(());
    }
    if parts.source_posture == Posture::Create && !recorded.preimage_demand().loci().is_empty() {
        return Err(());
    }
    for member in members {
        if let Member::OperationAftermath { contract, .. } = member
            && let Some(other) = contract
                .correction_binding()
                .and_then(|binding| binding.authored_identity())
            && other.parts().family == parts.family
            && !identity.same_family_as(other)
        {
            return Err(());
        }
    }
    Ok(())
}

fn require_role(
    binding: &ApplicationMutationBindingDescriptor,
    name: &str,
    entity: &str,
    posture: Posture,
) -> Result<(), ()> {
    let mut roles = binding
        .output_roles()
        .iter()
        .filter(|role| role.name() == name);
    let role = roles.next().ok_or(())?;
    (roles.next().is_none()
        && role.entity() == entity
        && role.posture() == posture
        && role.cardinality() == Cardinality::ExactlyOne)
        .then_some(())
        .ok_or(())
}

fn require_lifecycle(
    members: &[Member],
    operation: &str,
    entity: &str,
    posture: Posture,
) -> Result<(), ()> {
    members
        .iter()
        .any(|member| match member {
            Member::OperationProgram {
                operation: actual,
                target,
            } if actual == operation => match (posture, target) {
                (Posture::Create, Target::Create { entity: actual })
                | (Posture::Retire, Target::Delete { entity: actual }) => actual == entity,
                _ => false,
            },
            _ => false,
        })
        .then_some(())
        .ok_or(())
}
