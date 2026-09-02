pub(crate) struct UiAppearanceCellLookup {
    pub(crate) result: worth_ui_dsl::UiAppearanceDecisionResult,
    pub(crate) classes: Box<[worth_ui_dsl::UiAppearanceAxisClass]>,
    pub(crate) cell_ordinal: u32,
    pub(crate) visited: u32,
}

pub(crate) fn lookup(
    partition: &worth_ui_dsl::UiAppearanceDecisionPartition,
    vector: &super::super::super::state::UiAppearanceStateVector,
) -> Result<UiAppearanceCellLookup, super::UiAppearanceResolutionDenial> {
    let classes =
        partition
            .axes()
            .iter()
            .map(|axis| {
                vector.class(axis.axis()).ok_or(
                    super::UiAppearanceResolutionDenial::MissingStateAxis(axis.axis()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
    let mut visited = 0_u32;
    let cell = partition
        .cells()
        .iter()
        .enumerate()
        .find_map(|(index, cell)| {
            visited = visited.saturating_add(1);
            (cell.classes() == classes.as_ref()).then_some((index, cell))
        });
    let (index, cell) = cell.ok_or(super::UiAppearanceResolutionDenial::MissingDecisionCell(
        worth_ui_dsl::UiAppearanceAspect::Background,
    ))?;
    Ok(UiAppearanceCellLookup {
        result: cell.result().clone(),
        classes,
        cell_ordinal: u32::try_from(index)
            .expect("appearance decision cell count fits inspection ordinal")
            .saturating_add(1),
        visited,
    })
}
