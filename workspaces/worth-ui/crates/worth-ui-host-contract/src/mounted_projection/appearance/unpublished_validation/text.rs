use std::collections::HashSet;

use super::super::{
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFragmentIdentity,
    UiUnpublishedAppearanceFrameProjectionDenial,
};
use crate::UiMountedAppearanceMechanic;

pub(super) fn validate(
    fragment: &UiUnpublishedAppearanceFragment,
    frame: crate::UiMountedFrameIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    let successor_receipt = match fragment.identity {
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor, .. } => successor,
        UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(_)
        | UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { .. } => None,
    };
    if successor_receipt.is_none() {
        return if fragment.text_candidates.is_empty() {
            Ok(())
        } else {
            Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateRequiresNodeReceipt)
        };
    }
    let successor_receipt = successor_receipt.expect("checked above");
    let mut candidate_spans = HashSet::new();
    for candidate in &fragment.text_candidates {
        if candidate.frame() != frame {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateFrameMismatch);
        }
        if candidate.surface() != surface {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateSurfaceMismatch);
        }
        if candidate.binding() != fragment.surface_binding.binding() {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateBindingMismatch);
        }
        if candidate.capability_generation() != fragment.surface_binding.capability_generation() {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::CandidateCapabilityGenerationMismatch,
            );
        }
        if candidate.capability_profile_digest()
            != fragment.surface_binding.capability_profile_digest()
        {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::CandidateCapabilityProfileDigestMismatch,
            );
        }
        if candidate.content_generation() != fragment.presentation_affinity.content() {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateContentMismatch);
        }
        if candidate.node_receipt() != successor_receipt {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateReceiptMismatch);
        }
        for foreground in candidate.foregrounds() {
            let key = (candidate.node_receipt(), foreground.identity());
            if !candidate_spans.insert(key) {
                return Err(
                    UiUnpublishedAppearanceFrameProjectionDenial::ConflictingTextCandidate {
                        receipt: key.0,
                        span: key.1,
                    },
                );
            }
        }
    }
    for mechanic in fragment.work.successor().mechanics() {
        if let UiMountedAppearanceMechanic::TextForeground(text) = mechanic {
            if !candidate_spans.contains(&(text.node_receipt(), text.paint_span())) {
                return Err(
                    UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateSpanMismatch,
                );
            }
        }
    }
    Ok(())
}
