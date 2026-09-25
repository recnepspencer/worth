use crate::capability::{
    CapabilitySnapshot, ComponentId, MosaicLayoutCell, MosaicLayoutContract, MosaicLayoutDenial,
    MosaicResponsiveLayout, MosaicTrack, MosaicViewportWidthInterval, UiAuthoredComponentLayout,
    UiAuthoredLayoutCause, UiAuthoredLayoutDenial,
};
use worth_ui_dsl::{
    UiLayoutDeclaration, UiLayoutGrid, UiLayoutTrack, WorthUiSealedSemanticPackage,
};

/// Lowers each authored `layout` block through the Mosaic constructors, which
/// judge its tracks, cells, and intervals, then admits it only where it
/// restates its container's registered layout.
pub(super) fn admit_authored_layouts(
    package: &WorthUiSealedSemanticPackage,
    snapshot: &CapabilitySnapshot,
) -> Result<(), UiAuthoredLayoutDenial> {
    let authored = package
        .layout_declarations()
        .enumerate()
        .map(|(index, declaration)| lower_layout(index, declaration.declaration()))
        .collect::<Result<Vec<_>, _>>()?;
    snapshot.components().admit_authored_layouts(&authored)
}

fn lower_layout(
    declaration_index: usize,
    declaration: &UiLayoutDeclaration,
) -> Result<UiAuthoredComponentLayout, UiAuthoredLayoutDenial> {
    let denied = |cause| UiAuthoredLayoutDenial::new(declaration_index, cause);
    let mosaic = |cause| denied(UiAuthoredLayoutCause::Layout(cause));
    let container = ComponentId::new(declaration.container().as_str())
        .map_err(|_| denied(UiAuthoredLayoutCause::ContainerIdentityMalformed))?;
    let mut layout = MosaicResponsiveLayout::new(lower_grid(declaration.fallback(), &denied)?);
    for (interval, grid) in declaration.variants() {
        let interval = match interval.max() {
            None => MosaicViewportWidthInterval::at_least(interval.min()),
            Some(max) => {
                MosaicViewportWidthInterval::between(interval.min(), max).map_err(mosaic)?
            }
        };
        layout = layout
            .with_variant(interval, lower_grid(grid, &denied)?)
            .map_err(mosaic)?;
    }
    Ok(UiAuthoredComponentLayout::new(
        declaration_index,
        container,
        layout,
    ))
}

fn lower_grid(
    grid: &UiLayoutGrid,
    denied: &impl Fn(UiAuthoredLayoutCause) -> UiAuthoredLayoutDenial,
) -> Result<MosaicLayoutContract, UiAuthoredLayoutDenial> {
    let mosaic = |cause| denied(UiAuthoredLayoutCause::Layout(cause));
    let tracks = |tracks: &[UiLayoutTrack]| {
        tracks
            .iter()
            .map(|track| lower_track(*track))
            .collect::<Result<Vec<_>, _>>()
    };
    let mut contract = MosaicLayoutContract::grid(
        tracks(grid.columns()).map_err(mosaic)?,
        tracks(grid.rows()).map_err(mosaic)?,
    )
    .map_err(mosaic)?
    .with_gaps(grid.column_gap(), grid.row_gap())
    .with_padding(grid.inline_padding(), grid.block_padding());
    for (component, cell) in grid.members() {
        let member = ComponentId::new(component.as_str())
            .map_err(|_| denied(UiAuthoredLayoutCause::MemberIdentityMalformed))?;
        let cell = MosaicLayoutCell::spanning(
            cell.column(),
            cell.row(),
            cell.column_span(),
            cell.row_span(),
        )
        .map_err(mosaic)?;
        contract = contract.with_member(member, cell).map_err(mosaic)?;
    }
    Ok(contract)
}

const fn lower_track(track: UiLayoutTrack) -> Result<MosaicTrack, MosaicLayoutDenial> {
    match track {
        UiLayoutTrack::Fixed { extent } => MosaicTrack::fixed(extent),
        UiLayoutTrack::Flexible {
            weight,
            min,
            max: None,
        } => MosaicTrack::flex(weight, min),
        UiLayoutTrack::Flexible {
            weight,
            min,
            max: Some(max),
        } => MosaicTrack::bounded_flex(weight, min, max),
    }
}
