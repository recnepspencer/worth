use crate::capability::{
    MosaicRegionKindId, UiAuthoredScrollRegionCause, UiAuthoredScrollRegionClauses,
    UiAuthoredScrollRegionDenial, UiScrollAxisSupport, UiScrollChromeContract, UiScrollLineExtent,
};

use super::WorthUiSemanticHandoffEvidence;

impl WorthUiSemanticHandoffEvidence {
    /// The region-level scroll clauses the authored source states.
    ///
    /// A `scroll` block that states neither `line_extent` nor `chrome` declares
    /// an application-wide policy and names no region descriptor to change, so
    /// it contributes nothing here and its identity is never looked up.
    pub(super) fn authored_scroll_region_clauses(
        &self,
    ) -> Result<Vec<UiAuthoredScrollRegionClauses>, UiAuthoredScrollRegionDenial> {
        let mut clauses = Vec::new();
        for (declaration_index, declaration) in self.service_declarations().iter().enumerate() {
            let worth_ui_dsl::WorthUiServiceDeclarationMeaning::Scroll(scroll) =
                declaration.meaning()
            else {
                continue;
            };
            if scroll.line_extent_logical_points().is_none() && scroll.chrome().is_none() {
                continue;
            }
            clauses.push(lower_region_clauses(declaration_index, scroll)?);
        }
        Ok(clauses)
    }
}

fn lower_region_clauses(
    declaration_index: usize,
    scroll: &worth_ui_dsl::WorthUiScrollDeclaration,
) -> Result<UiAuthoredScrollRegionClauses, UiAuthoredScrollRegionDenial> {
    let denied = |cause| UiAuthoredScrollRegionDenial::new(declaration_index, cause);
    let region = MosaicRegionKindId::new(scroll.identity())
        .map_err(|_| denied(UiAuthoredScrollRegionCause::RegionIdentityMalformed))?;
    let line_extent = scroll
        .line_extent_logical_points()
        .map(UiScrollLineExtent::logical_points)
        .transpose()
        .map_err(|cause| denied(UiAuthoredScrollRegionCause::LineExtent(cause)))?;
    let chrome = scroll
        .chrome()
        .map(|chrome| lower_chrome(chrome, &denied))
        .transpose()?;
    Ok(UiAuthoredScrollRegionClauses::new(
        declaration_index,
        region,
        line_extent,
        chrome,
    ))
}

fn lower_chrome(
    chrome: &worth_ui_dsl::WorthUiScrollChromeDeclaration,
    denied: &impl Fn(UiAuthoredScrollRegionCause) -> UiAuthoredScrollRegionDenial,
) -> Result<UiScrollChromeContract, UiAuthoredScrollRegionDenial> {
    let axes = match chrome.axes() {
        worth_ui_dsl::WorthUiScrollChromeAxes::Inline => UiScrollAxisSupport::Inline,
        worth_ui_dsl::WorthUiScrollChromeAxes::Block => UiScrollAxisSupport::Block,
        worth_ui_dsl::WorthUiScrollChromeAxes::Both => UiScrollAxisSupport::Both,
    };
    let track = role_identity(chrome.track_role(), denied)?;
    let thumb = role_identity(chrome.thumb_role(), denied)?;
    UiScrollChromeContract::new(axes, track, thumb)
        .map_err(|cause| denied(UiAuthoredScrollRegionCause::Chrome(cause)))
}

fn role_identity(
    authored: &str,
    denied: &impl Fn(UiAuthoredScrollRegionCause) -> UiAuthoredScrollRegionDenial,
) -> Result<worth_ui_dsl::UiAppearanceRoleIdentity, UiAuthoredScrollRegionDenial> {
    worth_ui_dsl::UiAppearanceRoleIdentity::new(authored)
        .ok_or_else(|| denied(UiAuthoredScrollRegionCause::AppearanceRoleIdentityMalformed))
}
