//! Track-H admission scans actual tail effects before any World service exists.
use crate::{ApplicationHomeOpenRefusal, ReadmittedApplicationHome};
use worth_execution::ExecutionRequest;
mod relational;

pub(crate) fn admit(
    home: &ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeOpenRefusal> {
    relational::scan(home, request)
}
