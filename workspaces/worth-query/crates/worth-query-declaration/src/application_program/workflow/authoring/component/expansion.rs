use super::{AuthoredWorkflowComponent, ExpandedWorkflowComponent};
use crate::application_program::workflow::{
    ApplicationWorkflowComponentExpansion, ApplicationWorkflowConnection,
    ApplicationWorkflowExpandedConnectionProvenance, ApplicationWorkflowExpandedNodeProvenance,
    ApplicationWorkflowExpandedPortProvenance, ApplicationWorkflowNode, ApplicationWorkflowSpec,
};

use super::super::{
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentResource,
    ApplicationWorkflowDefinitionBuilder,
};

pub(in crate::application_program::workflow::authoring) fn expand<Spec>(
    builder: &mut ApplicationWorkflowDefinitionBuilder<Spec>,
    occurrence: &str,
    component: &AuthoredWorkflowComponent<Spec>,
) -> Result<ExpandedWorkflowComponent, ApplicationWorkflowAuthoringDenial>
where
    Spec: ApplicationWorkflowSpec,
{
    super::super::super::identity::require_identity_segment(occurrence)
        .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
    if builder
        .component_expansions
        .iter()
        .any(|expanded| expanded.occurrence_path() == occurrence)
    {
        return Err(
            ApplicationWorkflowAuthoringDenial::DuplicateComponentOccurrence(occurrence.to_owned()),
        );
    }
    require_expansion_capacity(builder, component)?;
    let expanded_nodes = component
        .nodes
        .iter()
        .map(|node| {
            Ok(ApplicationWorkflowNode::new(
                node.identity()
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
                node.kind().clone(),
            ))
        })
        .collect::<Result<Vec<_>, ApplicationWorkflowAuthoringDenial>>()?;
    if let Some(duplicate) = expanded_nodes.iter().find(|expanded| {
        builder
            .nodes
            .iter()
            .any(|existing| existing.identity() == expanded.identity())
    }) {
        return Err(ApplicationWorkflowAuthoringDenial::DuplicateNode(
            duplicate.identity().clone(),
        ));
    }
    let expanded_connections = component
        .connections
        .iter()
        .map(|connection| {
            Ok(ApplicationWorkflowConnection::new(
                connection
                    .source()
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
                connection
                    .target()
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
                connection.kind(),
            ))
        })
        .collect::<Result<Vec<_>, ApplicationWorkflowAuthoringDenial>>()?;
    let node_provenance = component
        .nodes
        .iter()
        .zip(&expanded_nodes)
        .map(|(authored, expanded)| {
            ApplicationWorkflowExpandedNodeProvenance::new(
                authored.identity().clone(),
                expanded.identity().clone(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let port_provenance = component
        .ports
        .iter()
        .map(|port| {
            Ok(ApplicationWorkflowExpandedPortProvenance::new(
                port.identity.clone(),
                port.direction,
                port.node.clone(),
                port.node
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
            ))
        })
        .collect::<Result<Vec<_>, ApplicationWorkflowAuthoringDenial>>()?
        .into_boxed_slice();
    let connection_provenance = component
        .connections
        .iter()
        .zip(&expanded_connections)
        .map(|(authored, expanded)| {
            ApplicationWorkflowExpandedConnectionProvenance::new(
                authored.source().clone(),
                authored.target().clone(),
                expanded.source().clone(),
                expanded.target().clone(),
                authored.kind(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let nested_expansions = component
        .component_expansions
        .iter()
        .map(|nested| {
            nested
                .qualified(occurrence)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)
        })
        .collect::<Result<Vec<_>, _>>()?;

    builder.nodes.extend(expanded_nodes);
    builder.connections.extend(expanded_connections);
    builder
        .component_expansions
        .push(ApplicationWorkflowComponentExpansion::new(
            component.identity.clone(),
            occurrence.to_owned(),
            port_provenance,
            node_provenance,
            connection_provenance,
        ));
    builder.component_expansions.extend(nested_expansions);
    Ok(ExpandedWorkflowComponent {
        component: component.identity.clone(),
        occurrence: occurrence.to_owned(),
        ports: component.ports.clone(),
        owner: std::sync::Arc::clone(&component.owner),
    })
}

fn require_expansion_capacity<Spec>(
    builder: &ApplicationWorkflowDefinitionBuilder<Spec>,
    component: &AuthoredWorkflowComponent<Spec>,
) -> Result<(), ApplicationWorkflowAuthoringDenial>
where
    Spec: ApplicationWorkflowSpec,
{
    require_total(
        ApplicationWorkflowComponentResource::ExpandedNodes,
        builder.nodes.len(),
        component.nodes.len(),
        usize::from(builder.limits.maximum_nodes()),
    )?;
    require_total(
        ApplicationWorkflowComponentResource::ExpandedConnections,
        builder.connections.len(),
        component.connections.len(),
        usize::from(builder.limits.maximum_connections()),
    )?;
    require_total(
        ApplicationWorkflowComponentResource::ComponentOccurrences,
        builder.component_expansions.len(),
        component.component_expansions.len().saturating_add(1),
        usize::from(builder.limits.maximum_nodes()),
    )?;

    let depth = usize::from(builder.limits.maximum_component_depth());
    let node_limit = usize::from(builder.limits.maximum_nodes()).saturating_mul(depth);
    let edge_limit = usize::from(builder.limits.maximum_connections()).saturating_mul(depth);
    let existing = provenance_counts(&builder.component_expansions);
    let nested = provenance_counts(&component.component_expansions);
    require_total(
        ApplicationWorkflowComponentResource::NodeProvenance,
        existing.0,
        component.nodes.len().saturating_add(nested.0),
        node_limit,
    )?;
    require_total(
        ApplicationWorkflowComponentResource::ConnectionProvenance,
        existing.1,
        component.connections.len().saturating_add(nested.1),
        edge_limit,
    )?;
    require_total(
        ApplicationWorkflowComponentResource::PortProvenance,
        existing.2,
        component.ports.len().saturating_add(nested.2),
        edge_limit,
    )
}

fn provenance_counts(
    expansions: &[ApplicationWorkflowComponentExpansion],
) -> (usize, usize, usize) {
    expansions.iter().fold((0, 0, 0), |counts, expansion| {
        (
            counts.0.saturating_add(expansion.nodes().len()),
            counts.1.saturating_add(expansion.connections().len()),
            counts.2.saturating_add(expansion.ports().len()),
        )
    })
}

fn require_total(
    resource: ApplicationWorkflowComponentResource,
    retained: usize,
    incoming: usize,
    maximum: usize,
) -> Result<(), ApplicationWorkflowAuthoringDenial> {
    if retained.saturating_add(incoming) > maximum {
        Err(
            ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                resource,
                maximum: u32::try_from(maximum).unwrap_or(u32::MAX),
            },
        )
    } else {
        Ok(())
    }
}
