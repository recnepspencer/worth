use super::{AuthoredWorkflowComponent, ComponentExpansionUsage, ExpandedWorkflowComponent};
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
    let incoming_usage = require_expansion_capacity(builder, component)?;
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
    builder.component_expansion_usage = ComponentExpansionUsage {
        occurrences: builder
            .component_expansion_usage
            .occurrences
            .saturating_add(incoming_usage.occurrences),
        maximum_depth: builder
            .component_expansion_usage
            .maximum_depth
            .max(incoming_usage.maximum_depth),
        node_provenance: builder
            .component_expansion_usage
            .node_provenance
            .saturating_add(incoming_usage.node_provenance),
        connection_provenance: builder
            .component_expansion_usage
            .connection_provenance
            .saturating_add(incoming_usage.connection_provenance),
        port_provenance: builder
            .component_expansion_usage
            .port_provenance
            .saturating_add(incoming_usage.port_provenance),
    };
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
) -> Result<ComponentExpansionUsage, ApplicationWorkflowAuthoringDenial>
where
    Spec: ApplicationWorkflowSpec,
{
    let incoming = ComponentExpansionUsage {
        occurrences: component.expansion_usage.occurrences.saturating_add(1),
        maximum_depth: component.expansion_usage.maximum_depth.saturating_add(1),
        node_provenance: component
            .expansion_usage
            .node_provenance
            .saturating_add(component.nodes.len()),
        connection_provenance: component
            .expansion_usage
            .connection_provenance
            .saturating_add(component.connections.len()),
        port_provenance: component
            .expansion_usage
            .port_provenance
            .saturating_add(component.ports.len()),
    };
    let limits = builder.limits.component_limits();
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
        builder.component_expansion_usage.occurrences,
        incoming.occurrences,
        usize::from(limits.maximum_occurrences()),
    )?;
    if incoming.maximum_depth > usize::from(limits.maximum_depth()) {
        return Err(
            ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                resource: ApplicationWorkflowComponentResource::ComponentDepth,
                maximum: u32::from(limits.maximum_depth()),
            },
        );
    }
    require_total(
        ApplicationWorkflowComponentResource::NodeProvenance,
        builder.component_expansion_usage.node_provenance,
        incoming.node_provenance,
        limits.maximum_node_provenance() as usize,
    )?;
    require_total(
        ApplicationWorkflowComponentResource::ConnectionProvenance,
        builder.component_expansion_usage.connection_provenance,
        incoming.connection_provenance,
        limits.maximum_connection_provenance() as usize,
    )?;
    require_total(
        ApplicationWorkflowComponentResource::PortProvenance,
        builder.component_expansion_usage.port_provenance,
        incoming.port_provenance,
        limits.maximum_port_provenance() as usize,
    )?;
    Ok(incoming)
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
