//! Durable declarations quote their private owner layouts without recreating them.
use super::*;
use worth_relational_contracts::internal_port::history::data::canonical_commit_envelope::schema_carrier::registrations::DurableEntitySchemaRegistration;
use worth_foundational::facade::PortableAspectContract;
use worth_relational_contracts::internal_port::schema::data::AspectExclusionGroupDeclaration;

pub(super) fn entity(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "kind_name" | "schema_id" => text(value, request)?,
                "declarations" => declarations(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn declarations(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let (mut aspects, mut identities, mut policies, mut exclusion_groups) = (0, 0, 0, 0);
    value.fields(request, |field, value| {
        match field {
            "aspects" => {
                aspects = aspects.max(value.elements(request, |value| {
                    value.fields(request, |field, value| {
                        add(
                            &mut bytes,
                            match field {
                                "binding" => super::patch::binding(value, request)?,
                                "contract" => contract(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })
                })?)
            }
            "identity_declarations" => {
                identities = identities.max(value.elements(request, |value| {
                    add(&mut bytes, super::identities::declaration(value, request)?)
                })?)
            }
            "merge_policy_declarations" => {
                policies = policies.max(value.elements(request, |value| {
                    add(
                        &mut bytes,
                        super::identities::policy_declaration(value, request)?,
                    )
                })?)
            }
            "exclusion_groups" => {
                exclusion_groups = exclusion_groups.max(value.elements(request, |value| {
                    add(&mut bytes, exclusion_group(value, request)?)
                })?)
            }
            _ => {}
        }
        Ok(())
    })?;
    let backing = DurableEntitySchemaRegistration::declaration_roster_storage_bytes(
        vector_capacity(aspects)?,
        vector_capacity(identities)?,
        vector_capacity(policies)?,
        vector_capacity(exclusion_groups)?,
    )
    .and_then(|bytes| u64::try_from(bytes).ok())
    .ok_or_else(malformed)?;
    add(
        &mut bytes,
        OwnerAllocation {
            retained: backing,
            transient: backing,
        },
    )?;
    Ok(bytes)
}
fn exclusion_group(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        match field {
            "group_id" => add(&mut bytes, text(value, request)?)?,
            "aspects" => {
                let members =
                    value.elements(request, |member| add(&mut bytes, text(member, request)?))?;
                let retained = AspectExclusionGroupDeclaration::member_tree_bytes(members)
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(malformed)?;
                let staging = vector_capacity(members)?
                    .checked_mul(size_of::<worth_foundational::facade::AspectKey>())
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(malformed)?;
                add(
                    &mut bytes,
                    OwnerAllocation {
                        retained,
                        transient: staging,
                    },
                )?;
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(bytes)
}

pub(super) fn contract(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "key" {
            add(&mut bytes, text(value, request)?)?;
        }
        if field == "shape" && value.is_map() {
            value.fields(request, |variant, value| {
                if variant == "Struct" {
                    let count = value.elements(request, |value| {
                        value.fields(request, |field, value| {
                            if field == "key" {
                                add(&mut bytes, text(value, request)?)?;
                            }
                            Ok(())
                        })
                    })?;
                    let backing = PortableAspectContract::decode_field_roster_storage_bytes(
                        vector_capacity(count)?,
                    )
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(malformed)?;
                    add(
                        &mut bytes,
                        OwnerAllocation {
                            retained: backing,
                            transient: backing,
                        },
                    )?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
