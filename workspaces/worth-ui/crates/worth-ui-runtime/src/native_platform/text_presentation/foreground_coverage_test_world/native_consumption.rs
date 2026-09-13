//! Native consumption of the coverage world and its host-surface geometry.
use super::{
    prepare_complete_semantic_text, CoverageWorld, UiMountedEventTimeDpiAuthority,
    UiNativeTextAtlasTransaction, UiNativeTextPresentationPreparation,
};
use std::rc::Rc;
use worth_ui_host_contract::*;

impl CoverageWorld {
    pub(in crate::native_platform::text_presentation) fn with_native<Output>(
        &self,
        attempt: UiMountedPresentationAttemptIdentity,
        operation: impl FnOnce(&UiMountedFrameConsumptionView<'_>) -> Output,
    ) -> (
        Output,
        super::super::rasterization::UiNativeTextRasterWorkReport,
    ) {
        let UiNativeTextPresentationPreparation::Prepared(prepared) =
            prepare_complete_semantic_text(
                self.fragment.text_candidates(),
                UiMountedEventTimeDpiAuthority::from_requirement(self.requirement).unwrap(),
                UiGlyphRasterLane::Ordinary,
                |id| (id == self.layout.identity()).then_some(self.layout.as_ref()),
            )
            .unwrap()
        else {
            panic!("complete text preparation denied");
        };
        let mut cache = worth_ui_text::UiGlyphRasterCache::default();
        let mut transaction = UiNativeTextAtlasTransaction::prepare(
            &prepared,
            |id| (id == self.layout.identity()).then_some(self.layout.as_ref()),
            &mut cache,
        )
        .unwrap();
        let resolver = Resolver(&self.layout);
        transaction.with_mounted_work(
            UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
            &[],
            |raster| {
                let UiHostProtocolNegotiation::Compatible(protocol) =
                    UiHostProtocolContract::current().negotiate()
                else {
                    panic!("current protocol denied");
                };
                let view = UiMountedFrameConsumptionView::from_inert_mechanics(
                    UiMountedFrameConsumptionInput {
                        authority: Rc::new(()),
                        host_session_identity: 41,
                        protocol,
                        capability_generation: self.requirement.capability_generation(),
                        capability_profile_digest: self.requirement.capability_profile_digest(),
                        attempt,
                        deadline: UiPresentationDeadline::at_tick(100),
                        requirement: self.requirement,
                        presentation_work: UiMountedPresentationWorkView::Unchanged(
                            &self.presentation,
                        ),
                        appearance_work: self.appearance_work.as_ref(),
                        qualified_text: &resolver,
                        text_raster_work: Some(raster),
                    },
                );
                operation(&view)
            },
        )
    }
}

pub(super) struct Resolver<'a>(&'a worth_ui_text::UiQualifiedTextLayout);
impl UiMountedQualifiedTextResolver for Resolver<'_> {
    fn resolve(
        &self,
        identity: UiQualifiedTextLayoutIdentity,
    ) -> Option<UiQualifiedTextLayoutView<'_>> {
        (identity == self.0.identity()).then(|| self.0.view())
    }
}
pub(super) fn bounds(value: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: value[0],
        y: value[1],
        width: value[2],
        height: value[3],
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
