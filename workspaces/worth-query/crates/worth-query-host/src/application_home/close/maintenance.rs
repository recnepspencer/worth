//! Retire the private Host operation inside its unchanged original Arc payload.
use super::*;
use crate::application_home::maintenance::HomeMaintenance;
use worth_query_execution::facade::application_home_integration::WorthQueryHomeMaintenance;

pub(super) fn retire(
    original: &mut dyn WorthQueryHomeMaintenance,
    home: &WorldSealedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeCloseDenial> {
    let original = (original as &mut dyn std::any::Any)
        .downcast_mut::<HomeMaintenance>()
        .ok_or(ApplicationHomeCloseDenial::UnsupportedHomeMaintenance)?;
    if original.retired_operation.is_some() {
        return Ok(());
    }
    let operation = original
        .operation
        .take()
        .expect("original Host operation remains until retirement");
    match home.retire_checkpoint_operation(operation, request) {
        Ok(retired) => {
            original.retired_operation = Some(retired);
            Ok(())
        }
        Err((operation, cause)) => {
            original.operation = Some(operation);
            Err(cause)
        }
    }
}
