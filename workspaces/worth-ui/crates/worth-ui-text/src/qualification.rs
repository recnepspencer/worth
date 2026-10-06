use std::sync::Arc;

use crate::{
    UiAdmittedTextParagraph, UiAnalyzedTextParagraph, UiFallbackTextParagraph,
    UiGlobalFontCollection, UiQualifiedTextLayout, UiQualifiedTextLayoutRequest,
    UiShapedTextParagraph, UiTextFallbackDenial, UiTextLayoutDenial,
    UiTextParagraphAdmissionDenial, UiTextParagraphAdmissionInput, UiTextShapingDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextQualificationDenial {
    Admission(UiTextParagraphAdmissionDenial),
    Fallback(UiTextFallbackDenial),
    Shaping(UiTextShapingDenial),
    Layout(UiTextLayoutDenial),
}

pub fn qualify_text_layout(
    input: UiTextParagraphAdmissionInput,
    fonts: Arc<UiGlobalFontCollection>,
) -> Result<UiQualifiedTextLayout, UiTextQualificationDenial> {
    UiQualifiedTextLayoutRequest::new(input, fonts).qualify()
}

pub(crate) fn qualify_request(
    request: UiQualifiedTextLayoutRequest,
) -> Result<UiQualifiedTextLayout, UiTextQualificationDenial> {
    let mut layout = qualify(request.clone(), QualificationPosture::Fresh)?;
    let source = Arc::new(crate::UiQualifiedTextReconstructionSource::new(
        request,
        layout.identity(),
    ));
    layout.attach_reconstruction_source(source);
    Ok(layout)
}

pub(crate) fn qualify_reconstruction(
    request: UiQualifiedTextLayoutRequest,
) -> Result<UiQualifiedTextLayout, UiTextQualificationDenial> {
    qualify(request, QualificationPosture::Reconstruction)
}

fn qualify(
    request: UiQualifiedTextLayoutRequest,
    posture: QualificationPosture,
) -> Result<UiQualifiedTextLayout, UiTextQualificationDenial> {
    let (input, fonts) = request.into_parts();
    let (admitted, _) = UiAdmittedTextParagraph::admit_from(input, &fonts, posture)
        .map_err(UiTextQualificationDenial::Admission)?;
    let analyzed = UiAnalyzedTextParagraph::analyze(admitted);
    let fallback = UiFallbackTextParagraph::select_with_posture(analyzed, fonts, posture)
        .map_err(UiTextQualificationDenial::Fallback)?;
    let shaped =
        UiShapedTextParagraph::shape(fallback).map_err(UiTextQualificationDenial::Shaping)?;
    UiQualifiedTextLayout::layout_with_posture(shaped, posture)
        .map_err(UiTextQualificationDenial::Layout)
}

#[derive(Clone, Copy)]
pub(crate) enum QualificationPosture {
    Fresh,
    Reconstruction,
}

impl QualificationPosture {
    pub(crate) const fn requires_current_collection(self) -> bool {
        matches!(self, Self::Fresh)
    }
}
