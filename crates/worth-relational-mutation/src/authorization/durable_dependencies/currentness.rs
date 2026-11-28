//! Compare an issued native source stamp with one exact commit snapshot.

use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
use crate::branch_port::snapshots::data::{SnapshotHandle, SnapshotHandleFieldsPort as _};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use crate::branch_port::visibility::materialization::read_records::projection::adjacency_revision_read::RelationalAdjacencyDirection;
use crate::branch_port::visibility::materialization::read_records::projection::creation::ProjectionInputs;
use crate::branch_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use crate::branch_port::visibility::snapshot_states::resolution::SnapshotReadDirectory;

use super::{
    capture, FieldDependency, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationTraversalDirection,
    DURABLE_DEPENDENCY_VERSION, MAXIMUM_AUTHORIZATION_DEPENDENCIES,
};

/// The native directory and projection share the same installed snapshot owner.
pub struct RelationalAuthorizationDependencySnapshots<'a> {
    pub projection: ProjectionInputs<'a>,
    pub directory: SnapshotReadDirectory<'a>,
}

impl RelationalAuthorizationDependencySnapshots<'_> {
    pub fn project(self, snapshot: &SnapshotHandle) -> Result<VisibilityProjectionView, Denial> {
        if (*snapshot.owner_runtime_instance_id()) != self.projection.runtime_identity {
            return Err(Denial::ForeignRuntime);
        }
        VisibilityProjectionView::from_snapshot(self.projection, self.directory, snapshot)
            .ok_or(Denial::SnapshotUnavailable)
    }
}

impl RelationalAuthorizationDurableDependencies {
    /// Checks retained source revisions without reconstructing the original policy.
    /// The caller must separately establish that the stamp was issued by its owner
    /// and that the current authorization still admits the requested action.
    /// Cost is O(retained dependencies), bounded by the native wire contract.
    pub fn source_revisions_match<'a>(
        &self,
        snapshots: impl Into<RelationalAuthorizationDependencySnapshots<'a>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, Denial> {
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        if self.owned.data.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        self.check_wire_limits(execution)?;
        let view = snapshots.into().project(snapshot)?;
        self.revisions_match_in_projection(execution, &view)
    }

    fn revisions_match_in_projection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &VisibilityProjectionView,
    ) -> Result<bool, Denial> {
        if !view.is_exact_basis() {
            return Err(Denial::InexactSnapshot);
        }
        let mut remaining = MAXIMUM_AUTHORIZATION_DEPENDENCIES;
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        if capture::capture_entity(
            execution,
            view,
            self.owned.data.principal.entity,
            &mut remaining,
        )? != self.owned.data.principal
            || capture::capture_entity(
                execution,
                view,
                self.owned.data.scope.entity,
                &mut remaining,
            )? != self.owned.data.scope
        {
            return Ok(false);
        }
        for path in &self.owned.data.paths {
            execution
                .consult()
                .map_err(|stop| Denial::Execution(stop.into()))?;
            capture::consume(&mut remaining)?;
            if let Some(witness) = &path.witness {
                capture::consume_many(&mut remaining, witness.len())?;
            }
            for entity in &path.entities {
                execution
                    .consult()
                    .map_err(|stop| Denial::Execution(stop.into()))?;
                if capture::capture_entity(execution, view, entity.entity, &mut remaining)?
                    != *entity
                {
                    return Ok(false);
                }
            }
            for relation in &path.relations {
                execution
                    .consult()
                    .map_err(|stop| Denial::Execution(stop.into()))?;
                if capture::capture_relation(execution, view, relation.relation, &mut remaining)?
                    != *relation
                {
                    return Ok(false);
                }
            }
            for adjacency in &path.adjacencies {
                execution
                    .consult()
                    .map_err(|stop| Denial::Execution(stop.into()))?;
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
                        execution,
                        adjacency.entity,
                        adjacency.relation_kind,
                        direction,
                        1,
                    )
                    .map_err(capture::adjacency_denial)?
                    .revision();
                if revision != adjacency.revision {
                    return Ok(false);
                }
            }
            for field in &path.fields {
                capture::consume(&mut remaining)?;
                if !field_revision_matches(execution, view, field)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

fn field_revision_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    field: &FieldDependency,
) -> Result<bool, Denial> {
    let _locator_storage = super::storage::locator_storage(execution, &field.aspect, &field.field)?;
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new(&field.aspect).ok_or(Denial::MalformedWire)?,
        CanonicalFieldPath::single(FieldKey::new(&field.field).ok_or(Denial::MalformedWire)?),
    );
    Ok(view
        .entity_field_revision(execution, field.entity, &locator)
        .map_err(Denial::Execution)?
        .ok_or(Denial::DependencyUnavailable)?
        == field.revision)
}

impl<'a, Source> From<&'a std::sync::Arc<Source>> for RelationalAuthorizationDependencySnapshots<'a>
where
    Self: From<&'a Source>,
{
    fn from(source: &'a std::sync::Arc<Source>) -> Self {
        <Self as From<&'a Source>>::from(source.as_ref())
    }
}
