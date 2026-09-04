pub(crate) fn resolve(
    aspect: worth_ui_dsl::UiAppearanceAspect,
    partition: &worth_ui_dsl::UiAppearanceDecisionPartition,
    vector: &super::super::super::state::UiAppearanceStateVector,
    theme: &super::super::super::theme::UiThemeResolutionView,
) -> Result<super::super::UiResolvedAppearanceAspect, super::UiAppearanceResolutionFailure> {
    let lookup = super::cell_lookup::lookup(partition, vector, aspect)
        .map_err(|denial| super::UiAppearanceResolutionFailure::without_theme_work(denial))?;
    finish(aspect, lookup, theme)
}

pub(crate) fn resolve_backdrop(
    aspect: worth_ui_dsl::UiAppearanceAspect,
    partition: &worth_ui_dsl::UiAppearanceDecisionPartition,
    theme: &super::super::super::theme::UiThemeResolutionView,
) -> Result<super::super::UiResolvedAppearanceAspect, super::UiAppearanceResolutionFailure> {
    let lookup = super::cell_lookup::lookup_without_state(partition, aspect)
        .map_err(|denial| super::UiAppearanceResolutionFailure::without_theme_work(denial))?;
    finish(aspect, lookup, theme)
}

fn finish(
    aspect: worth_ui_dsl::UiAppearanceAspect,
    lookup: super::cell_lookup::UiAppearanceCellLookup,
    theme: &super::super::super::theme::UiThemeResolutionView,
) -> Result<super::super::UiResolvedAppearanceAspect, super::UiAppearanceResolutionFailure> {
    let slot = lookup.result.slot().ok_or_else(|| {
        super::UiAppearanceResolutionFailure::without_theme_work(
            super::UiAppearanceResolutionDenial::MissingDecisionCell(aspect),
        )
    })?;
    let resolved = theme
        .resolve(slot, lookup.result.value_kind())
        .map_err(|failure| {
            super::UiAppearanceResolutionFailure::with_theme_work(
                super::UiAppearanceResolutionDenial::ThemeResolution(failure.denial()),
                failure.work().theme_slots_compared(),
            )
        })?;
    let support = super::support::for_aspect(aspect, theme);
    let provenance = super::provenance::from_theme(&resolved, theme);
    let digest = super::provenance::semantic_digest(
        aspect,
        lookup.classes.as_ref(),
        resolved.value(),
        &provenance,
        support,
    );
    Ok(super::super::UiResolvedAppearanceAspect::new(
        aspect,
        lookup.classes,
        lookup.cell_ordinal,
        resolved.value(),
        provenance,
        support,
        digest,
        lookup.visited,
        resolved.work().theme_slots_compared(),
    ))
}
