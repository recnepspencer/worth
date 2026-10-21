//! Output delivery closes over the program's actual root topology and source occurrence.
use super::*;
pub(super) fn validate_action_deliveries<Schema, Program>(
    actions: &[ApplicationActionDeclaration],
) -> Result<(), ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    let roots = Program::Outputs::root_graph_types();
    for action in actions {
        let Some(delivery) = action.delivery() else {
            continue;
        };
        require_identity(delivery.identity())?;
        let discovered = matches!(
            delivery,
            super::super::ApplicationActionDeliveryDeclaration::Discovered(_)
        );
        let Some(delivery) = delivery.output() else {
            if action.required_output_source() {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::InvalidActionDelivery,
                    action.binding(),
                ));
            }
            continue;
        };
        if let Some(read) = delivery.read() {
            require_identity(read.identity())?;
        }
        let connection = delivery.root_connection();
        if discovered != delivery.is_discovered()
            || !action.required_output_source()
            || delivery.batch_items() == Some(0)
            || !roots.contains(&delivery.root_type())
            || connection.source_instance() != action.composition_instance()
            || connection.source_feature() != action.feature()
            || delivery
                .read()
                .is_some_and(|read| read.limits().maximum_results() == 0)
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::InvalidActionDelivery,
                action.binding(),
            ));
        }
    }
    Ok(())
}
