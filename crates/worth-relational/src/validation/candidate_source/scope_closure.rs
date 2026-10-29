//! Complete reverse scope discovery from the applied Native mutation effect.
use std::collections::{BTreeMap, BTreeSet};

use super::program::ManagedIndexSourceProgram;
use crate::authority::mutation::AdjacencyDelta;
use crate::derived_relation::evaluation::{evaluate_position, SourceRead};
use crate::derived_relation::{DerivedRelationData, DerivedRelationDenial as Definition};
use crate::derived_relation::{
    DerivedRelationReadDenial as Denial, DerivedRelationReads, StructuralSource,
};
use crate::graph_member::NativeMemberKey;
use crate::identity::data::EntityId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::RecordRef;
use crate::validation::data::{StructuralReadError, StructuralRelationView};

/// Support includes negative inverse reads. Scope discovery does not depend
/// on public sessions, retained provider roots, or positive query matches.
pub(crate) struct PreparedManagedIndexScopeClosure {
    pub(crate) scopes: BTreeMap<EntityId, BTreeSet<EntityId>>,
    pub(crate) support: Vec<DerivedRelationData>,
}

pub(crate) fn discover(
    registry: &RelationalSchemaRegistry,
    program: &ManagedIndexSourceProgram,
    records: &[RecordRef],
    adjacency: &[AdjacencyDelta],
    before: &StructuralRelationView<'_>,
    after: &StructuralRelationView<'_>,
) -> Result<PreparedManagedIndexScopeClosure, Denial> {
    before.require_derived_relation(&program.0.path)?;
    after.require_derived_relation(&program.0.path)?;
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
        &program.0.path,
        adjacency,
        records,
        &mut old,
        &mut new,
    )?;
    let mut closure = PreparedManagedIndexScopeClosure {
        scopes: BTreeMap::new(),
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
    // Lifecycle pullback intentionally excludes unchanged-kind scalar writes.
    // Route every actually changed row endpoint, including owner-induced edits,
    // through the same inverse Source program on BOTH real physical sides.
    for record in records {
        new.checkpoint(1)?;
        let RecordRef::Entity(id) = *record else {
            continue;
        };
        for source in [&mut old, &mut new] {
            match source.entity_kind(id) {
                Ok(kind) if kind == program.row_kind() => {
                    rows.insert(NativeMemberKey::Entity(id));
                }
                Ok(kind) if kind == program.scope_kind() => {
                    closure.scopes.entry(id).or_default();
                }
                Ok(_) | Err(Denial::Definition(Definition::RecordUnavailable)) => {}
                Err(cause) => return Err(cause),
            }
        }
    }
    let position = program
        .0
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
                &program.0.path,
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
            source.retain(size_of::<DerivedRelationData>())?;
            closure.support.push(data);
        }
    }
    new.checkpoint(0)?;
    Ok(closure)
}

/// Only lifecycle absence is data; audience, storage, budget and original
/// request stops retain their exact causes.
struct LifecycleSource<'a, 'b>(StructuralSource<'a, 'b>);
impl SourceRead for LifecycleSource<'_, '_> {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        self.0.checkpoint(units)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        self.0.retain(bytes)
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
        probe: crate::derived_relation::DerivedRelationAdjacency,
    ) -> Result<Vec<crate::validation::data::StructuralRelationRecord>, Denial> {
        self.0.adjacency(probe)
    }
    fn reads(&self) -> DerivedRelationReads {
        self.0.reads()
    }
}
