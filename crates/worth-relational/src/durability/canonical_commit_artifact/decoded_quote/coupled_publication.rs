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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_stages_vector_still_refuses_noncanonical_payload() {
        #[derive(serde::Serialize)]
        enum Sources {
            Stages(&'static str),
        }
        #[derive(serde::Serialize)]
        struct Malformed {
            sources: Sources,
            participants: Vec<()>,
        }
        let bytes = rmp_serde::to_vec_named(&Malformed {
            sources: Sources::Stages("not an entity vector"),
            participants: Vec::new(),
        })
        .unwrap();
        let request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationSource::new().token(),
            None,
        );
        assert!(matches!(
            quote(BorrowedPayload(&bytes), &ExecutionRequest::serial(&request)),
            Err(Refusal::NonCanonicalPayload)
        ));
    }
}
