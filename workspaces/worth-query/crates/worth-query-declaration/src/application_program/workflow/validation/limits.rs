use crate::application_program::workflow::{ApplicationWorkflowSpec, AuthoredWorkflowDefinition};

use super::{
    denial, ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
    ValidationWorkMeter,
};

pub(super) fn enforce<Spec>(
    authored: &AuthoredWorkflowDefinition<Spec>,
    work: &mut ValidationWorkMeter,
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
    let effects = authored.nodes.iter().fold(0_usize, |effects, node| {
        work.visit_limit_node();
        effects + usize::from(node.kind().is_effect())
    });
    if effects > usize::from(limits.maximum_effects()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::EffectLimitExceeded,
            effects.to_string(),
        ));
    }
    let component_limits = limits.component_limits();
    if authored.component_expansions.len() > usize::from(component_limits.maximum_occurrences()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::ComponentOccurrenceLimitExceeded,
            authored.component_expansions.len().to_string(),
        ));
    }
    let mut node_provenance = 0_usize;
    let mut connection_provenance = 0_usize;
    let mut port_provenance = 0_usize;
    for expansion in &authored.component_expansions {
        work.visit_provenance_record();
        let depth = expansion.occurrence_path().split('/').count();
        if depth > usize::from(component_limits.maximum_depth()) {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::ComponentDepthExceeded,
                expansion.occurrence_path(),
            ));
        }
        node_provenance = node_provenance.saturating_add(expansion.nodes().len());
        connection_provenance = connection_provenance.saturating_add(expansion.connections().len());
        port_provenance = port_provenance.saturating_add(expansion.ports().len());
    }
    if node_provenance > component_limits.maximum_node_provenance() as usize {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::NodeProvenanceLimitExceeded,
            node_provenance.to_string(),
        ));
    }
    if connection_provenance > component_limits.maximum_connection_provenance() as usize {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::ConnectionProvenanceLimitExceeded,
            connection_provenance.to_string(),
        ));
    }
    if port_provenance > component_limits.maximum_port_provenance() as usize {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::PortProvenanceLimitExceeded,
            port_provenance.to_string(),
        ));
    }
    Ok(())
}
