//! A new isolated event changes only its own membership, never the old prefix.
use super::*;
use crate::authority::mutation::AdjacencyDeltaKind;
use crate::derived_relation::DerivedRelationAdjacency;

pub(super) fn discover(
    program: &ManagedIndexSourceProgram,
    records: &[RecordRef],
    adjacency: &[AdjacencyDelta],
    before: &StructuralRelationView<'_>,
    after: &StructuralRelationView<'_>,
) -> Result<Option<PreparedManagedIndexScopeClosure>, Denial> {
    let Some(recipe) = program.0.ordered else {
        return Ok(None);
    };
    let mut old = LifecycleSource(StructuralSource {
        view: before,
        reads: DerivedRelationReads::default(),
    });
    let mut new = LifecycleSource(StructuralSource {
        view: after,
        reads: DerivedRelationReads::default(),
    });
    let mut created = BTreeSet::new();
    let mut scope = None;
    for record in records {
        new.checkpoint(1)?;
        let RecordRef::Entity(entity) = *record else {
            continue;
        };
        match old.entity_kind(entity) {
            Ok(_) => return Ok(None),
            Err(Denial::Definition(Definition::RecordUnavailable)) => {}
            Err(cause) => return Err(cause),
        }
        let kind = new.entity_kind(entity)?;
        new.retain(size_of::<EntityId>() * 5)?;
        created.insert(entity);
        if kind == program.scope_kind() && scope.replace(entity).is_some() {
            return Ok(None);
        }
    }
    let Some(scope) = scope else { return Ok(None) };
    let kinds: BTreeSet<_> = program
        .physical_sources()
        .map(|&(kind, _, _)| kind)
        .collect();
    for delta in adjacency
        .iter()
        .filter(|delta| kinds.contains(&delta.kind_id))
    {
        new.checkpoint(1)?;
        match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
                if created.contains(&source)
                    && (delta.kind_id != recipe.predecessor || !created.contains(&target)) => {}
            _ => return Ok(None),
        }
    }
    // Native permits dangling audit links. New lifecycle alone must not hide
    // an old positive or negative membership/projection dependency on this ID.
    for &entity in &created {
        for &relation in &kinds {
            for outgoing in [false, true] {
                if !old
                    .adjacency(DerivedRelationAdjacency {
                        anchor: entity,
                        relation,
                        outgoing,
                    })?
                    .is_empty()
                {
                    return Ok(None);
                }
            }
        }
    }
    let changed = created
        .into_iter()
        .filter(|entity| *entity != scope)
        .collect();
    new.retain(size_of::<(EntityId, BTreeSet<EntityId>)>())?;
    Ok(Some(PreparedManagedIndexScopeClosure {
        scopes: BTreeMap::from([(scope, changed)]),
        support: Vec::new(),
    }))
}
