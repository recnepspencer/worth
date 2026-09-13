//! Issue mounted region name bindings from sealed structural declarations.
use super::{WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};
use std::collections::BTreeSet;

impl WorthUiSemanticPackageSealingState {
    pub(super) fn seal_region_bindings(&mut self) -> Result<(), crate::WorthUiDslCompileReport> {
        let mut names = BTreeSet::new();
        for module in self.modules.values() {
            for declaration in &module.declarations {
                let structure = match declaration {
                    WorthUiSemanticDeclaration::Component(block)
                    | WorthUiSemanticDeclaration::Surface(block) => &block.structure,
                    _ => continue,
                };
                let mut pending = structure.root_regions().iter().collect::<Vec<_>>();
                while let Some(region) = pending.pop() {
                    names.insert(region.region_id_text().to_owned());
                    pending.extend(region.child_regions());
                }
            }
        }
        // Typed Rust Backdrops can carry unnamed region identities. Preserve
        // those exact values and do not reuse them for a newly named region.
        let reserved = self
            .backdrops
            .iter()
            .filter_map(|(backdrop, _)| match backdrop.extent() {
                crate::UiBackdropExtentBasis::PresentedMosaicRegion { region, .. } => {
                    Some(region.value())
                }
                crate::UiBackdropExtentBasis::SurfaceViewport(_) => None,
            })
            .max()
            .unwrap_or(0);
        self.overlay_declaration_bindings
            .bind_declared_regions(&names, reserved)
            .map_err(|()| {
                super::super::overlay_identity_resolution::resolver_report(
                    super::super::overlay_identity_resolution::ResolutionDenial::Capacity(
                        "mosaic region",
                    ),
                    self.provenance_table.first(),
                )
            })
    }
}
