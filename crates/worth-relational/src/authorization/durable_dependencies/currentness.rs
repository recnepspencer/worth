//! Compare an issued native source stamp with one exact commit snapshot.

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::materialization::read_records::RelationalAdjacencyDirection;

use super::{
    capture, FieldDependency, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationTraversalDirection,
    DURABLE_DEPENDENCY_VERSION, MAXIMUM_AUTHORIZATION_DEPENDENCIES,
};

impl RelationalAuthorizationDurableDependencies {
    /// Checks retained source revisions without reconstructing the original policy.
    /// The caller must separately establish that the stamp was issued by its owner
    /// and that the current authorization still admits the requested action.
    /// Cost is O(retained dependencies), bounded by the native wire contract.
    pub fn source_revisions_match(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Denial> {
        if self.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        self.bounded_wire_bytes()?;
        if snapshot.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        let view = runtime
            .read_truth()
            .project_snapshot(snapshot)
            .ok_or(Denial::SnapshotUnavailable)?;
        if !view.is_exact_basis() {
            return Err(Denial::InexactSnapshot);
        }
        let mut remaining = MAXIMUM_AUTHORIZATION_DEPENDENCIES;
        if capture::capture_entity(&view, self.principal.entity, &mut remaining)? != self.principal
            || capture::capture_entity(&view, self.scope.entity, &mut remaining)? != self.scope
        {
            return Ok(false);
        }
        for path in &self.paths {
            capture::consume(&mut remaining)?;
            if let Some(witness) = &path.witness {
                capture::consume_many(&mut remaining, witness.len())?;
            }
            for entity in &path.entities {
                if capture::capture_entity(&view, entity.entity, &mut remaining)? != *entity {
                    return Ok(false);
                }
            }
            for relation in &path.relations {
                if capture::capture_relation(&view, relation.relation, &mut remaining)? != *relation
                {
                    return Ok(false);
                }
            }
            for adjacency in &path.adjacencies {
                capture::consume(&mut remaining)?;
                let direction = match adjacency.direction {
                    RelationalAuthorizationTraversalDirection::Forward => {
                        RelationalAdjacencyDirection::Outgoing
                    }
                    RelationalAuthorizationTraversalDirection::Reverse => {
                        RelationalAdjacencyDirection::Incoming
                    }
                };
                let revision = view
                    .bounded_adjacency_structural_revision(
                        adjacency.entity,
                        adjacency.relation_kind,
                        direction,
                        1,
                    )
                    .map_err(|_| Denial::DependencyUnavailable)?
                    .revision();
                if revision != adjacency.revision {
                    return Ok(false);
                }
            }
            for field in &path.fields {
                capture::consume(&mut remaining)?;
                if !field_revision_matches(&view, field)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

fn field_revision_matches(
    view: &crate::visibility::materialization::read_records::VisibilityProjectionView<'_>,
    field: &FieldDependency,
) -> Result<bool, Denial> {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new(&field.aspect).ok_or(Denial::MalformedWire)?,
        CanonicalFieldPath::single(FieldKey::new(&field.field).ok_or(Denial::MalformedWire)?),
    );
    Ok(view
        .entity_field_revision(field.entity, &locator)
        .ok_or(Denial::DependencyUnavailable)?
        == field.revision)
}
