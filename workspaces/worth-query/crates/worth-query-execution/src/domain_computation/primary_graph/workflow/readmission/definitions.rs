//! Canonical retained descriptors and membership on the exact selected branch.
use super::*;
use crate::domain_computation::primary_graph::workflow::definition::{
    read_recorded_definition, RecordedWorkflowDefinition,
};

pub(super) fn read(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    population: &WorkflowPopulation,
    current: CanonicalDigestId,
    history: &BTreeSet<CanonicalDigestId>,
) -> Result<BTreeMap<EntityId, RecordedWorkflowDefinition>, Denial> {
    for lineage in population.entities(layout.lineage.entity_kind) {
        fields
            .protocol(
                *lineage,
                layout.lineage.entity_kind,
                &layout.lineage.protocol_version,
            )
            .map_err(Denial::Record)?;
        if fields
            .text(*lineage, layout.lineage.entity_kind, &layout.lineage.spec)
            .map_err(Denial::Record)?
            .bytes()
            .is_empty()
        {
            return Err(Denial::CorruptRecord {
                entity: *lineage,
                kind: layout.lineage.entity_kind,
            });
        }
    }
    let mut result = BTreeMap::new();
    let mut nodes = BTreeSet::new();
    let mut connections = BTreeSet::new();
    for definition in population.entities(layout.definition.entity_kind) {
        let data = read_recorded_definition(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            *definition,
        )
        .map_err(|cause| Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause)))?;
        for revision in [data.published_under(), data.effective_revision()] {
            if !history.contains(&revision) {
                return Err(Denial::UnattributedProgram {
                    entity: *definition,
                    revision,
                });
            }
        }
        if population.parent(layout.lineage_definition_relation, *definition)? != data.lineage() {
            return Err(Denial::CorruptRecord {
                entity: *definition,
                kind: layout.definition.entity_kind,
            });
        }
        let mut paths = BTreeSet::new();
        for node in data.nodes() {
            if !paths.insert(node.path()) {
                return Err(Denial::CorruptRecord {
                    entity: node.entity(),
                    kind: layout.node.entity_kind,
                });
            }
            if population.parent(layout.definition_node_relation, node.entity())? != *definition
                || !nodes.insert(node.entity())
            {
                return Err(Denial::CorruptRecord {
                    entity: node.entity(),
                    kind: layout.node.entity_kind,
                });
            }
        }
        for connection in population.outgoing(layout.definition_connection_relation, *definition) {
            if population.parent(layout.definition_connection_relation, *connection)? != *definition
                || !connections.insert(*connection)
            {
                return Err(Denial::CorruptRecord {
                    entity: *connection,
                    kind: layout.connection.entity_kind,
                });
            }
        }
        result.insert(*definition, data);
    }
    for (kind, seen) in [
        (layout.node.entity_kind, &nodes),
        (layout.connection.entity_kind, &connections),
    ] {
        if let Some(entity) = population
            .entities(kind)
            .iter()
            .find(|entity| !seen.contains(entity))
        {
            return Err(Denial::CorruptRecord {
                entity: *entity,
                kind,
            });
        }
    }
    for lineage in population.entities(layout.lineage.entity_kind) {
        if let Some(definition) = population.one(layout.current_definition_relation, *lineage)? {
            let data = result.get(&definition).ok_or(Denial::CorruptRecord {
                entity: definition,
                kind: layout.definition.entity_kind,
            })?;
            if data.lineage() != *lineage {
                return Err(Denial::CorruptRecord {
                    entity: definition,
                    kind: layout.definition.entity_kind,
                });
            }
            if data.effective_revision() != current {
                return Err(Denial::CurrentProgramMismatch { entity: definition });
            }
        }
    }
    Ok(result)
}
