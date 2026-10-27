//! Validate installed scalar prefix types and authoritative emitted rows at the pinned basis.
use super::*;
use crate::indexes::projected_field_values::IndexProjectionSource;
use worth_foundational::facade::AspectShape;

pub(super) fn prefix(
    projection: &crate::runtime::VisibilityProjectionView,
    request: &BoundedRelatedEntityPrefixLookupRequest<'_>,
    prepared: &PreparedRelatedLookup<'_>,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    charge(meter, 1)?;
    let source = IndexProjectionSource::exact(projection).ok_or(Denial::InvalidPrefix)?;
    let ordering = prepared.contract.ordering();
    if request.prefix.len() > ordering.len() {
        return Err(Denial::InvalidPrefix);
    }
    let _catalog = source.aspect_plans().ok_or(Denial::InvalidPrefix)?;

    let plan = source
        .entity_aspect_plan(request.ordered.child_kind())
        .ok_or(Denial::InvalidPrefix)?;
    for (value, field) in request.prefix.iter().zip(ordering) {
        let mut expected = None;
        for binding in &plan.executable_bindings {
            charge(
                meter,
                1 + binding.aspect_key().as_str().len() as u64
                    + field.locator().aspect().aspect_key().as_str().len() as u64,
            )?;
            if binding.aspect_key() != field.locator().aspect().aspect_key() {
                continue;
            }
            match binding.contract.shape() {
                AspectShape::Scalar(kind) => {
                    let [key] = field.locator().field_path().fields() else {
                        return Err(Denial::InvalidPrefix);
                    };
                    charge(meter, 1 + key.as_str().len() as u64)?;
                    if binding.targets_entity_scalar_field(key) {
                        expected = Some(*kind)
                    }
                }
                AspectShape::Struct(shape) => {
                    if !matches!(
                        &binding.target,
                        crate::schema::data::AspectBinding::EntityField { .. }
                    ) {
                        return Err(Denial::InvalidPrefix);
                    }
                    let [key] = field.locator().field_path().fields() else {
                        return Err(Denial::InvalidPrefix);
                    };
                    for declared in shape.fields() {
                        charge(
                            meter,
                            1 + key.as_str().len() as u64 + declared.key().as_str().len() as u64,
                        )?;
                        if declared.key() == key {
                            expected = Some(declared.value_type())
                        }
                    }
                }
                _ => return Err(Denial::InvalidPrefix),
            }
            break;
        }
        charge(meter, 1)?;
        if !matches!(expected,Some(kind) if kind==value.value_family()) {
            return Err(Denial::InvalidPrefix);
        }
        // Canonical scalar validation after family validation; no arbitrary nested prefix payload.
        charge(meter, 1 + value.semantic_byte_width() as u64)?;
        if !value.has_canonical_representation() {
            return Err(Denial::InvalidPrefix);
        }
    }
    Ok(())
}
pub(super) fn row(
    read: &RelationalPinnedRead,
    projection: &crate::runtime::VisibilityProjectionView,
    request: &BoundedRelatedEntityPrefixLookupRequest<'_>,
    prepared: &PreparedRelatedLookup<'_>,
    entry: &RelatedEntityOrderingEntry,
    execution: &ExecutionRequest<'_, '_>,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    charge(meter, 1)?;
    let corrupt = || {
        Denial::Lookup(BoundedRelatedEntityOrderedLookupDenial::new(
            BoundedRelatedEntityOrderedLookupDenialKind::CorruptIndexEntries,
            request.ordered.index_id(),
        ))
    };
    if entry.ordering_values().len() != prepared.contract.ordering().len() {
        return Err(corrupt());
    }
    let definition = &prepared.generation;

    let selected = read
        .selected_index(definition.index_id)
        .ok_or(Denial::InvalidPrefix)?;
    let DerivedIndexKind::RelatedEntityOrdering {
        relation_kind,
        parent_endpoint,
        child_kind,
        ..
    } = &selected.definition.kind
    else {
        return Err(Denial::InvalidPrefix);
    };
    charge(
        meter,
        projection
            .exact_relation_metadata_read_work_bound()
            .ok_or(Denial::ExactBasisRequired)?,
    )?;
    let relation = projection
        .read_exact_relation_metadata(execution, entry.relation_id())
        .map_err(|denial| match denial {
            crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                Denial::ExactBasisRequired
            }
            crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                Denial::Read(cause)
            }
        })?
        .ok_or_else(&corrupt)?;
    let (parent, child) = match parent_endpoint {
        RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
        RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
    };
    if relation.kind_id != *relation_kind
        || parent != request.ordered.parent_entity_id()
        || child != entry.child_entity_id()
    {
        return Err(corrupt());
    }
    // This probe is independent of ordering iteration, including complete-range reads.
    charge(
        meter,
        projection
            .exact_entity_state_read_work_bound()
            .ok_or(Denial::ExactBasisRequired)?,
    )?;
    let actual_child = projection
        .read_exact_entity_state(execution, child, |metadata, _| metadata)
        .map_err(|denial| match denial {
            crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                Denial::ExactBasisRequired
            }
            crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                Denial::Read(cause)
            }
        })?
        .ok_or_else(&corrupt)?;
    if actual_child.kind_id != *child_kind {
        return Err(corrupt());
    }
    for (field, value) in prepared
        .contract
        .ordering()
        .iter()
        .zip(entry.ordering_values())
    {
        let matched = super::super::field_candidate_comparison::borrowed_matches(
            execution,
            request.ordered.index_id(),
            projection,
            child,
            field.locator(),
            value.value(),
        )
        .map_err(Denial::FieldLookup)?;
        if !matches!(matched,Some((_,kind,true)) if kind==*child_kind) {
            return Err(corrupt());
        }
    }
    Ok(())
}
