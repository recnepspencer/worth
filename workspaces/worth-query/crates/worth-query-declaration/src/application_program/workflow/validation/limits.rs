use crate::application_program::workflow::{ApplicationWorkflowSpec, AuthoredWorkflowDefinition};

use super::{denial, ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind};

pub(super) fn enforce<Spec>(
    authored: &AuthoredWorkflowDefinition<Spec>,
) -> Result<(), ApplicationWorkflowValidationDenial>
where
    Spec: ApplicationWorkflowSpec,
{
    let limits = authored.limits;
    if authored.nodes.len() > usize::from(limits.maximum_nodes()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::NodeLimitExceeded,
            authored.nodes.len().to_string(),
        ));
    }
    if authored.connections.len() > usize::from(limits.maximum_connections()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::ConnectionLimitExceeded,
            authored.connections.len().to_string(),
        ));
    }
    let effects = authored
        .nodes
        .iter()
        .filter(|node| node.kind().is_effect())
        .count();
    if effects > usize::from(limits.maximum_effects()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::EffectLimitExceeded,
            effects.to_string(),
        ));
    }
    for node in &authored.nodes {
        let depth = node.identity().as_str().split('/').count();
        if depth > usize::from(limits.maximum_component_depth()) {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::ComponentDepthExceeded,
                node.identity().as_str(),
            ));
        }
    }
    Ok(())
}
