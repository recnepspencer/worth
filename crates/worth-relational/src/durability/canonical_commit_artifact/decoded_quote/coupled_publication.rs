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
