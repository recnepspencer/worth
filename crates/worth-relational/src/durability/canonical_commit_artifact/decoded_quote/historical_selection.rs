//! Quote the selected basis, genuine publications and explicit absent routes.
use super::*;
use crate::history::data::{CanonicalHistoricalOutputSelection, CanonicalRequiredOutputOrigin};
use std::sync::Arc;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    optional(value, |value| {
        let mut bytes = shared_root::<CanonicalHistoricalOutputSelection>()?;
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "selected_basis" => super::branches::descriptor(value, request)?,
                    "publications" => {
                        vector::<Arc<CanonicalRequiredOutputOrigin>>(value, request, |origin| {
                            super::required_output::origin(origin, request)
                        })?
                    }
                    "absences" => vector_with_storage(
                        value,
                        request,
                        CanonicalHistoricalOutputSelection::absence_decode_storage,
                        |absence| {
                            let mut bytes = OwnerAllocation::default();
                            absence.fields(request, |field, value| match field {
                                "meaning" => add(
                                    &mut bytes,
                                    super::required_output::meaning(value, request)?,
                                ),
                                "publication" | "generation" => Ok(()),
                                _ => Err(Refusal::NonCanonicalPayload),
                            })?;
                            Ok(bytes)
                        },
                    )?,
                    _ => return Err(Refusal::NonCanonicalPayload),
                },
            )
        })?;
        Ok(bytes)
    })
}
