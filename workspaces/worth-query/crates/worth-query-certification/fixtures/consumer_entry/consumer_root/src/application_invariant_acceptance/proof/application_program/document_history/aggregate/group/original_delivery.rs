//! A grouped fixed member retains its complete historical delivery contract.
use super::*;

pub(super) fn validate(
    delivery: &WorthQueryClosedFixedDelivery<
        ConsumerSchema,
        Original,
        FixedProgram,
        Root,
        FixedAction,
    >,
    request: &Request<'_, '_, '_>,
) -> Result<(), WorthQueryOperationDeliveryDenial> {
    // compiled-example: begin
    let original = request.at(delivery.settlement().observation());
    delivery.deliver_current_at(&original)
    // compiled-example: end
}
