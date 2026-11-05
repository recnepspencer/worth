use sha2::{Digest, Sha256};

/// Derives the portable commitment to one immutable observed-source footprint.
pub(super) fn checkpoint_source_identity(
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &super::super::WorthQueryObservedSourceFootprint,
) -> [u8; 32] {
    match checkpoint_source_identity_admitted(query, parameters, footprint, &mut |_| {
        Ok::<(), std::convert::Infallible>(())
    }) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

/// The same sole encoding, with original admission before each actual hash input.
pub(super) fn checkpoint_source_identity_admitted<E>(
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &super::super::WorthQueryObservedSourceFootprint,
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<[u8; 32], E> {
    let mut digest = Sha256::new();
    let mut emit = |bytes: &[u8]| {
        admit(bytes.len())?;
        digest.update(bytes);
        Ok(())
    };
    emit(b"worth-query:checkpoint-observed-source:v2")?;
    emit(query)?;
    emit(parameters)?;
    append_footprint(&mut emit, footprint)?;
    Ok(digest.finalize().into())
}
fn append_footprint<E>(
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
    footprint: &super::super::WorthQueryObservedSourceFootprint,
) -> Result<(), E> {
    entity(emit, footprint.root)?;
    emit(&[u8::from(footprint.complete)])?;
    length(emit, footprint.entities.len())?;
    for observed in &footprint.entities {
        entity(emit, *observed)?;
    }
    length(emit, footprint.aspects.len())?;
    for aspect in &footprint.aspects {
        entity(emit, aspect.entity)?;
        text(emit, &aspect.entity_name)?;
        text(emit, aspect.aspect.as_str())?;
        text(emit, aspect.field.as_str())?;
        emit(&aspect.contract_revision.0.to_be_bytes())?;
        field_revision(emit, aspect.native_revision)?;
    }
    length(emit, footprint.adjacencies.len())?;
    for adjacency in &footprint.adjacencies {
        entity(emit, adjacency.anchor)?;
        emit(&adjacency.relation_kind.as_u32().to_be_bytes())?;
        emit(&[match adjacency.direction {
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
        }])?;
        optional_u64(emit, adjacency.native_revision.map(|revision| revision.0))?;
        length(emit, adjacency.comparison_work_limit)?;
        length(emit, adjacency.endpoints.len())?;
        for endpoint in &adjacency.endpoints {
            entity(emit, *endpoint)?;
        }
    }
    emit(&[u8::from(footprint.root_selection.is_some())])?;
    if let Some(selection) = &footprint.root_selection {
        emit(&selection.identity())?;
    }
    Ok(())
}
fn entity<E>(
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
    value: worth_relational::facade::identity::EntityId,
) -> Result<(), E> {
    emit(&value.partition_value().to_be_bytes())?;
    emit(&value.local_slot_value().to_be_bytes())?;
    emit(&value.generation_value().to_be_bytes())
}
fn text<E>(emit: &mut impl FnMut(&[u8]) -> Result<(), E>, value: &str) -> Result<(), E> {
    length(emit, value.len())?;
    emit(value.as_bytes())
}
fn length<E>(emit: &mut impl FnMut(&[u8]) -> Result<(), E>, value: usize) -> Result<(), E> {
    emit(
        &u64::try_from(value)
            .expect("source footprint lengths fit the canonical u64 carrier")
            .to_be_bytes(),
    )
}
fn optional_u64<E>(
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
    value: Option<u64>,
) -> Result<(), E> {
    emit(&[u8::from(value.is_some())])?;
    emit(&value.unwrap_or_default().to_be_bytes())
}
fn field_revision<E>(
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
    revision: Option<worth_relational::facade::runtime::RelationalFieldRevision>,
) -> Result<(), E> {
    emit(&[u8::from(revision.is_some())])?;
    if let Some(revision) = revision {
        emit(&revision.version().0.to_be_bytes())?;
        emit(&[match revision.presence() {
            worth_relational::facade::runtime::RelationalFieldPresence::Present => 1,
            worth_relational::facade::runtime::RelationalFieldPresence::Absent => 0,
        }])?;
    }
    Ok(())
}
