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
    if identity.member_owner().is_some() {
        return Err(());
    }
    let live = identity.live_member_owner().is_some();
    require_live_owner(members, identity, recorded)?;
    let owned_members = identity
        .co_anchors()
        .iter()
        .any(|member| member.member_owner().is_some());
    validate_identity(
        members,
        source,
        inverse,
        identity,
        recorded,
        false,
        owned_members,
    )?;
    for (index, anchor) in identity.co_anchors().iter().enumerate() {
        // Every family of a live-owner inverse names its own live owner; no
        // anchor of the declaration is created or retired with its members.
        if anchor.live_member_owner().is_some() != live
            || (live && anchor.member_owner().is_some())
            || (!owned_members
                && !live
                && anchor.parts().source_posture != identity.parts().source_posture)
            || !anchor.co_anchors().is_empty()
            || anchor.parts().source_role == identity.parts().source_role
            || anchor.parts().inverse_role == identity.parts().inverse_role
            || identity.co_anchors()[..index].iter().any(|prior| {
                prior.parts().source_role == anchor.parts().source_role
                    || prior.parts().inverse_role == anchor.parts().inverse_role
            })
        {
            return Err(());
        }
        validate_identity(
            members,
            source,
            inverse,
            anchor,
            recorded,
            true,
            owned_members,
        )?;
        require_live_owner(members, anchor, recorded)?;
        if let Some(owner) = anchor.member_owner() {
            if anchor.is_optional() {
                return Err(());
            }
            let Member::Field { entity: parent, .. } = identity.parts().logical_key.as_ref() else {
                return Err(());
            };
            let Member::Field { entity: child, .. } = anchor.parts().logical_key.as_ref() else {
                return Err(());
            };
            if owner.from() != child || owner.to() != parent || !members.iter().any(|member| matches!(member,
                Member::Relation { relation, from, to, integrity }
                if relation == owner.relation() && from == child && to == parent
                    && integrity.cardinality.source_min == Some(1) && integrity.cardinality.source_max == Some(1))) {
                return Err(());
            }
        }
    }
    Ok(())
}

fn validate_identity(
    members: &[Member],
    source: &ApplicationMutationBindingDescriptor,
    inverse: &ApplicationMutationBindingDescriptor,
    identity: &worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection,
    recorded: &worth_query_declaration::facade::application_aftermath::PortableRecordedInverse,
    co_anchor: bool,
    owned_members: bool,
) -> Result<(), ()> {
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
    if identity.member_owner().is_some() || identity.live_member_owner().is_some() {
        require_family(source, &parts.source_role, entity, parts.source_posture)?;
        require_family(inverse, &parts.inverse_role, entity, inverse_posture)?;
    } else {
        require_role(
            source,
            &parts.source_role,
            entity,
            parts.source_posture,
            co_anchor && identity.is_optional(),
        )?;
        require_role(
            inverse,
            &parts.inverse_role,
            entity,
            inverse_posture,
            co_anchor && identity.is_optional(),
        )?;
    }
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
    if parts.source_posture == Posture::Create
        && !(co_anchor && owned_members)
        && identity.live_member_owner().is_none()
        && !recorded.preimage_demand().loci().is_empty()
    {
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

/// A live owner's member family is a source of one exactly-one owner relation
/// whose rows the recorded inverse retains.
fn require_live_owner(
    members: &[Member],
    identity: &worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection,
    recorded: &worth_query_declaration::facade::application_aftermath::PortableRecordedInverse,
) -> Result<(), ()> {
    let Some(owner) = identity.live_member_owner() else {
        return Ok(());
    };
    let Member::Field { entity, .. } = identity.parts().logical_key.as_ref() else {
        return Err(());
    };
    (owner.from() == entity
        && recorded.preimage_demand().relations().contains(owner)
        && members.iter().any(|member| matches!(member,
            Member::Relation { relation, from, to, integrity }
            if relation == owner.relation() && from == owner.from() && to == owner.to()
                && integrity.cardinality.source_min == Some(1) && integrity.cardinality.source_max == Some(1))))
    .then_some(())
    .ok_or(())
}

fn require_family(
    binding: &ApplicationMutationBindingDescriptor,
    prefix: &str,
    entity: &str,
    posture: Posture,
) -> Result<(), ()> {
    let mut families = binding
        .output_role_families()
        .iter()
        .filter(|family| family.prefix() == prefix);
    let family = families.next().ok_or(())?;
    (families.next().is_none() && !prefix.is_empty() && family.entity() == entity
        && family.minimum() == 0 && family.postures() == worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet::one(posture))
        .then_some(()).ok_or(())
}

fn require_role(
    binding: &ApplicationMutationBindingDescriptor,
    name: &str,
    entity: &str,
    posture: Posture,
    optional: bool,
) -> Result<(), ()> {
    let mut roles = binding
        .output_roles()
        .iter()
        .filter(|role| role.name() == name);
    let role = roles.next().ok_or(())?;
    (roles.next().is_none()
        && role.entity() == entity
        && role.posture() == posture
        && role.cardinality()
            == if optional {
                Cardinality::AtMostOne
            } else {
                Cardinality::ExactlyOne
            })
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
