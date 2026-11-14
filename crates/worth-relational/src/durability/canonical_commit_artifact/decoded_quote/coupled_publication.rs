//! Quote every genuine participant and source event in the current coupled grammar.
use super::*;
use crate::history::data::CanonicalCoupledAuthoredPublication;

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
                                "Retirement" => {
                                    corrective_sources(event, request, super::cascades::authored)?
                                }
                                "Restoration" => corrective_sources(
                                    event,
                                    request,
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
    prefix_quote: impl Fn(
        BorrowedPayload<'_>,
        &ExecutionRequest<'_, '_>,
    ) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let (mut prefix, mut later) = (false, false);
    value.fields(request, |field, value| {
        let nested = match field {
            "prefix" if !prefix => {
                prefix = true;
                prefix_quote(value, request)?
            }
            "later" if !later => {
                later = true;
                vector::<crate::identity::data::EntityId>(value, request, |_| {
                    Ok(OwnerAllocation::default())
                })?
            }
            _ => return Err(Refusal::NonCanonicalPayload),
        };
        add(&mut bytes, nested)
    })?;
    if !prefix || !later {
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
        let sources = vector::<crate::identity::data::EntityId>(value, request, |_| {
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
