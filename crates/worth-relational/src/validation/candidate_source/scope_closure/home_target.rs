//! Build a target closure from every real proposed scope, without relabelling
//! the predecessor's definitions or treating its durable data as empty.
use super::*;

pub(in crate::validation::candidate_source) fn discover_home_target(
    registry: &RelationalSchemaRegistry,
    program: &ManagedIndexSourceProgram,
    records: &[RecordRef],
    after: &StructuralRelationView<'_>,
) -> Result<PreparedManagedIndexScopeClosure, Denial> {
    after.require_derived_relation(&program.0.path)?;
    let mut source = LifecycleSource(StructuralSource {
        view: after,
        reads: DerivedRelationReads::default(),
    });
    let mut closure = PreparedManagedIndexScopeClosure {
        scopes: BTreeMap::new(),
        support: Vec::new(),
    };
    for record in records {
        source.checkpoint(1)?;
        let RecordRef::Entity(entity) = record else {
            continue;
        };
        match source.entity_kind(*entity) {
            Ok(kind) if kind == program.scope_kind() => {}
            Ok(_) | Err(Denial::Definition(Definition::RecordUnavailable)) => continue,
            Err(cause) => return Err(cause),
        }
        if closure.scopes.contains_key(entity) {
            continue;
        }
        source.retain(size_of::<(EntityId, BTreeSet<EntityId>)>())?;
        let position = program
            .0
            .path
            .parts()
            .nodes
            .len()
            .checked_sub(1)
            .ok_or(Definition::InvalidExpression)?;
        let data = evaluate_position(
            registry,
            &program.0.path,
            position,
            false,
            (*entity).into(),
            &mut source,
        )?;
        let mut rows = BTreeSet::new();
        for endpoint in data.endpoints() {
            let row = endpoint.entity().ok_or(Definition::InvalidMemberDomain)?;
            source.retain(size_of::<EntityId>())?;
            rows.insert(row);
        }
        closure.scopes.insert(*entity, rows);
        crate::derived_relation::support::push(&mut source, &mut closure.support, data)?;
    }
    source.checkpoint(0)?;
    Ok(closure)
}
