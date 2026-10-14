//! Fixed delivery closes over the program's actual root topology and source occurrence.
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
        let Some(delivery) = delivery.fixed_output() else {
            if action.required_output_source() {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::InvalidActionDelivery,
                    action.binding(),
                ));
            }
            continue;
        };
        require_identity(delivery.read().identity())?;
        let connection = delivery.root_connection();
        if !action.required_output_source()
            || !roots.contains(&delivery.root_type())
            || connection.source_instance() != action.composition_instance()
            || connection.source_feature() != action.feature()
            || delivery.read().limits().maximum_results() == 0
            || delivery.read().limits().maximum_work() == Some(0)
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::InvalidActionDelivery,
                action.binding(),
            ));
        }
    }
    Ok(())
}
