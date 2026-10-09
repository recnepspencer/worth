use super::*;

pub(super) fn borrowed_matches<E>(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
    prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
) -> Result<Option<(EntityId, KindId, bool)>, Stop<E>> {
    let read = view
        .exact_entity_state_read_work_bound()
        .ok_or(Stop::ExactBasisRequired)?;
    let navigation = view
        .exact_entity_state_read_navigation_work_bound()
        .ok_or(Stop::ExactBasisRequired)?;
    charge(
        prepare,
        read.checked_sub(navigation)
            .ok_or(Stop::AccountingOverflow)?,
        0,
    )?;
    prepare(SelectedIndexReadWork::OrderedNavigation(navigation), 0).map_err(Stop::Admission)?;
    view.with_exact_entity_state(entity, |metadata, state| {
        let matches = state_matches(state, locator, expected, prepare)?;
        Ok((metadata.entity_id, metadata.kind_id, matches))
    })
    .map_err(|_| Stop::ExactBasisRequired)?
    .transpose()
}

fn state_matches<E>(
    state: Option<&AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
    prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
) -> Result<bool, Stop<E>> {
    let Some(state) = state else { return Ok(false) };
    let aspect = locator.aspect().aspect_key();
    admit_text_descent(prepare, state.aspects().len(), aspect.as_str().len())?;
    let Some(validated) = state.get(aspect) else {
        return Ok(false);
    };
    let actual = match validated.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(value) => {
            let [field] = locator.field_path().fields() else {
                return Ok(false);
            };
            admit_text_descent(prepare, value.len(), field.as_str().len())?;
            value.get(field)
        }
    };
    let Some(actual) = actual else {
        return Ok(false);
    };
    let comparison = actual
        .semantic_byte_width()
        .max(expected.semantic_byte_width());
    charge(
        prepare,
        width(comparison)?
            .checked_add(1)
            .ok_or(Stop::AccountingOverflow)?,
        0,
    )?;
    Ok(actual == expected)
}

fn admit_text_descent<E>(
    prepare: &mut impl FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
    entries: usize,
    text: usize,
) -> Result<(), Stop<E>> {
    let path = btree_navigation_work(entries, 0)?;
    let all = btree_navigation_work(entries, text)?;
    charge(
        prepare,
        all.checked_sub(path).ok_or(Stop::AccountingOverflow)?,
        0,
    )?;
    prepare(SelectedIndexReadWork::OrderedNavigation(path), 0).map_err(Stop::Admission)
}
