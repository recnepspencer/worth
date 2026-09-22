//! What the mechanic refuses when raw input contradicts the qualified layout
//! it claims to complete.
use std::sync::Arc;

use super::{assert_denial, fixture};
use crate::{
    UiMountedRgba8, UiMountedSemanticTextCompletionDenial, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity,
};

#[test]
fn raw_text_cannot_impersonate_a_different_qualified_layout() {
    let mut input = fixture();
    input.text = Arc::from("UPDATED");
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::QualifiedLayoutSourceMismatch,
    );
}

#[test]
fn foreground_spans_must_match_the_canonical_layout_itemization() {
    let mut input = fixture();
    input.foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
        crate::UiTextOriginalRange::new(0, 5).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
    )]);
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::ForegroundSpanMismatch,
    );
}
