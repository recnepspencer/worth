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
                                "Retirement" => super::cascades::authored(event, request)?,
                                "Restoration" => super::cascades::restoration(event, request)?,
                                "Edit" => OwnerAllocation::default(),
                                "Stages" => vector::<crate::identity::data::EntityId>(
                                    event,
                                    request,
                                    |_| Ok(OwnerAllocation::default()),
                                )?,
                                "RetirementWithLaterSources" => {
                                    corrective_sources(event, request, super::cascades::authored)?
                                }
                                "RestorationWithLaterSources" => corrective_sources(
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
                let mut present = false;
                let backing = vector::<crate::identity::data::EntityId>(value, request, |_| {
                    present = true;
                    Ok(OwnerAllocation::default())
                })?;
                if !present {
                    return Err(Refusal::NonCanonicalPayload);
                }
                backing
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
