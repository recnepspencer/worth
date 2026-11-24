//! Quote every genuine participant and source event in the current coupled grammar.
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::coupled_authored::CanonicalCoupledAuthoredPublication;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    optional(value, |value| {
        let mut bytes = shared_root::<CanonicalCoupledAuthoredPublication>()?;
        value.fields(request, |field, value| {
            let nested = match field {
                "sources" => {
                    let mut sources = OwnerAllocation::default();
                    value.fields(request, |variant, event| {
                        add(
                            &mut sources,
                            match variant {
                                "Edit" => edit_sources(event, request)?,
                                "Retirement" => corrective_sources(
                                    event,
                                    request,
                                    "later",
                                    super::cascades::authored,
                                )?,
                                "Restoration" => corrective_sources(
                                    event,
                                    request,
                                    "later",
                                    super::cascades::restoration,
                                )?,
                                "MemberEditRetirement" => corrective_sources(
                                    event,
                                    request,
                                    "sources",
                                    super::cascades::authored,
                                )?,
                                "MemberEditRestoration" => corrective_sources(
                                    event,
                                    request,
                                    "sources",
                                    super::cascades::restoration,
                                )?,
                                _ => return Err(Refusal::NonCanonicalPayload),
                            },
                        )
                    })?;
                    sources
                }
                "participants" => vector_with_storage(
                    value,
                    request,
                    CanonicalCoupledAuthoredPublication::participant_decode_storage,
                    |part| {
                        let mut bytes = OwnerAllocation::default();
                        part.fields(request, |field, value| {
                            add(
                                &mut bytes,
                                match field {
                                    "origin" => super::required_output::origin(value, request)?,
                                    "retirement" => super::cascades::producer(value, request)?,
                                    "participation" => OwnerAllocation::default(),
                                    _ => return Err(Refusal::NonCanonicalPayload),
                                },
                            )
                        })?;
                        Ok(bytes)
                    },
                )?,
                _ => return Err(Refusal::NonCanonicalPayload),
            };
            add(&mut bytes, nested)
        })?;
        Ok(bytes)
    })
}

fn corrective_sources(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    source_field: &'static str,
    prefix_quote: impl Fn(
        BorrowedPayload<'_>,
        &ExecutionRequest<'_, '_>,
    ) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let (mut prefix, mut later) = (false, false);
    let mut source_seen = false;
    value.fields(request, |field, value| {
        let nested = match field {
            "prefix" if !prefix => {
                prefix = true;
                prefix_quote(value, request)?
            }
            field if field == source_field && !later => {
                later = true;
                vector::<
                    worth_relational_contracts::internal_port::identity::data::record_ids::EntityId,
                >(value, request, |_| {
                    source_seen = true;
                    Ok(OwnerAllocation::default())
                })?
            }
            _ => return Err(Refusal::NonCanonicalPayload),
        };
        add(&mut bytes, nested)
    })?;
    if !prefix || !later || (source_field == "sources" && !source_seen) {
        return Err(Refusal::NonCanonicalPayload);
    }
    Ok(bytes)
}

fn edit_sources(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let (mut field_seen, mut prefix_seen) = (false, false);
    value.fields(request, |field, value| {
        if field != "sources" || field_seen {
            return Err(Refusal::NonCanonicalPayload);
        }
        field_seen = true;
        let sources = vector::<
            worth_relational_contracts::internal_port::identity::data::record_ids::EntityId,
        >(value, request, |_| {
            prefix_seen = true;
            Ok(OwnerAllocation::default())
        })?;
        add(&mut bytes, sources)
    })?;
    if !prefix_seen {
        return Err(Refusal::NonCanonicalPayload);
    }
    Ok(bytes)
}
