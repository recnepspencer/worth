//! Complete reverse scope discovery from the applied Native mutation effect.
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgramFieldsPort as _;

use super::program::ManagedIndexSourceProgram;
use crate::authority::mutation::AdjacencyDelta;
use crate::derived_relation::definition::DerivedRelationDenial as Definition;
use crate::derived_relation::evaluation::{evaluate_position, SourceRead};
use crate::derived_relation::read::DerivedRelationData;
use crate::derived_relation::read::DerivedRelationReads;
use crate::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use crate::derived_relation::structural::StructuralSource;
use crate::graph_member::NativeMemberKey;
use crate::identity::data::EntityId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::custom_rule::structural_views::StructuralRelationView;
use crate::validation::data::structural_read::StructuralReadError;

mod home_target;
mod ordered_birth;
mod projections;
pub(super) use home_target::discover_home_target;

/// Support includes negative inverse reads. Scope discovery does not depend
/// on public sessions, retained provider roots, or positive query matches.
pub(crate) struct PreparedManagedIndexScopeClosure {
    pub(crate) scopes: BTreeMap<EntityId, BTreeSet<EntityId>>,
    /// Sorted, deduplicated row-kind and projection-endpoint entity IDs whose
    /// own entity record changed, or `None` on a target-only rebuild.
    pub(crate) changed_occurrence_records: Option<Vec<EntityId>>,
    pub(crate) support: Vec<DerivedRelationData>,
}

fn is_occurrence_dependency_kind(
    program: &ManagedIndexSourceProgram,
    kind: crate::identity::data::KindId,
) -> bool {
    kind == program.row_kind()
        || program
            .projection_routes()
            .iter()
            .any(|route| route.endpoint_kind() == kind)
}

fn push_changed_occurrence_record(
    source: &mut LifecycleSource<'_, '_>,
    records: &mut Vec<EntityId>,
    entity: EntityId,
) -> Result<(), Denial> {
    crate::derived_relation::source_buffer::reserve(source, records, 1)?;
    records.push(entity);
    Ok(())
}

pub(crate) fn discover(
    registry: &RelationalSchemaRegistry,
    program: &ManagedIndexSourceProgram,
    records: &[RecordRef],
    adjacency: &[AdjacencyDelta],
    before: &StructuralRelationView<'_>,
    after: &StructuralRelationView<'_>,
) -> Result<PreparedManagedIndexScopeClosure, Denial> {
    before.require_derived_relation(&(*program.owner_field_0()).path)?;
    after.require_derived_relation(&(*program.owner_field_0()).path)?;
    if let Some(birth) = ordered_birth::discover(program, records, adjacency, before, after)? {
        return Ok(birth);
    }
    let mut old = LifecycleSource(StructuralSource {
        view: before,
        reads: DerivedRelationReads::default(),
    });
    let mut new = LifecycleSource(StructuralSource {
        view: after,
        reads: DerivedRelationReads::default(),
    });
    let affected = crate::derived_relation::affected::collect(
        registry,
        &(*program.owner_field_0()).path,
        adjacency,
        records,
        &mut old,
        &mut new,
    )?;
    let mut closure = PreparedManagedIndexScopeClosure {
        scopes: BTreeMap::new(),
        changed_occurrence_records: None,
        support: affected.support,
    };
    for root in affected.roots {
        let NativeMemberKey::Entity(scope) = root else {
            return Err(Definition::InvalidMemberDomain.into());
        };
        new.retain(size_of::<(EntityId, BTreeSet<EntityId>)>())?;
        closure.scopes.entry(scope).or_default();
    }
    let mut rows = affected.targets;
    projections::pullback(
        registry,
        program,
        records,
        adjacency,
        [&mut old, &mut new],
        &mut closure,
        &mut rows,
    )?;
    // Lifecycle pullback intentionally excludes unchanged-kind scalar writes.
    // Route every actually changed row endpoint, including owner-induced edits,
    // through the same inverse Source program on BOTH real physical sides, and
    // record changed entity IDs in the occurrence-dependency kind domain.
    let mut changed_occurrence_records = Vec::new();
    for record in records {
        new.checkpoint(1)?;
        let RecordRef::Entity(id) = *record else {
            continue;
        };
        let mut is_dependency_record = false;
        for source in [&mut old, &mut new] {
            match source.entity_kind(id) {
                Ok(kind) => {
                    if kind == program.row_kind() {
                        rows.insert(NativeMemberKey::Entity(id));
                    } else if kind == program.scope_kind() {
                        closure.scopes.entry(id).or_default();
                    }
                    if is_occurrence_dependency_kind(program, kind) {
                        is_dependency_record = true;
                    }
                }
                Err(Denial::Definition(Definition::RecordUnavailable)) => {}
                Err(cause) => return Err(cause),
            }
        }
        if is_dependency_record {
            push_changed_occurrence_record(&mut new, &mut changed_occurrence_records, id)?;
        }
    }
    changed_occurrence_records.sort_unstable();
    changed_occurrence_records.dedup();
    closure.changed_occurrence_records = Some(changed_occurrence_records);
    let position = (*program.owner_field_0())
        .path
        .parts()
        .nodes
        .len()
        .checked_sub(1)
        .ok_or(Definition::InvalidExpression)?;
    for row in rows {
        let NativeMemberKey::Entity(row) = row else {
            return Err(Definition::InvalidMemberDomain.into());
        };
        for source in [&mut old, &mut new] {
            match source.entity_kind(row) {
                Err(Denial::Definition(Definition::RecordUnavailable)) => continue,
                Err(cause) => return Err(cause),
                Ok(_) => {}
            }
            let data = evaluate_position(
                registry,
                &(*program.owner_field_0()).path,
                position,
                true,
                row.into(),
                source,
            )?;
            for root in data.endpoints() {
                let NativeMemberKey::Entity(scope) = root else {
                    return Err(Definition::InvalidMemberDomain.into());
                };
                source.retain(size_of::<(EntityId, EntityId)>())?;
                closure.scopes.entry(*scope).or_default().insert(row);
            }
            crate::derived_relation::support::push(source, &mut closure.support, data)?;
        }
    }
    new.checkpoint(0)?;
    Ok(closure)
}

/// Only lifecycle absence is data; audience, storage, budget and original
/// request stops retain their exact causes.
struct LifecycleSource<'a, 'b>(StructuralSource<'a, 'b>);
impl SourceRead for LifecycleSource<'_, '_> {
    fn admit_definition(
        &mut self,
        registry: &RelationalSchemaRegistry,
        handle: &crate::derived_relation::definition::DerivedRelationHandle,
    ) -> Result<(), Denial> {
        self.0.admit_definition(registry, handle)
    }
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        self.0.checkpoint(units)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        self.0.retain(bytes)
    }
    fn check_scratch_peak(&mut self, bytes: usize) -> Result<(), Denial> {
        self.0.check_scratch_peak(bytes)
    }
    fn entity_kind(&mut self, id: EntityId) -> Result<crate::identity::data::KindId, Denial> {
        match self.0.view.entity_kind(id) {
            Ok(kind) => {
                self.0.reads.entity_records += 1;
                Ok(kind)
            }
            Err(StructuralReadError::RecordUnavailable) => {
                Err(Definition::RecordUnavailable.into())
            }
            Err(cause) => Err(Denial::Structural(self.0.view.read_budget().refusal(cause))),
        }
    }
    fn adjacency(
        &mut self,
        probe: crate::derived_relation::read::DerivedRelationAdjacency,
    ) -> Result<Vec<crate::validation::data::structural_read::StructuralRelationRecord>, Denial>
    {
        self.0.adjacency(probe)
    }
    fn reads(&self) -> DerivedRelationReads {
        self.0.reads()
    }
}
