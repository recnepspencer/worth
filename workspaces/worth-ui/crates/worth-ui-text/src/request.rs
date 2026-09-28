use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::UiTextParagraphAdmissionInput;
pub use worth_ui_host_contract::UiQualifiedTextLayoutRequestIdentity;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiTextProfileGeneration, UiTextScaleGeneration,
};

/// Names a request at every width. Requests that differ only in width share
/// it, so a layout fitted for one of them can be found for the others.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiQualifiedTextReflowKey([u8; 32]);

impl UiQualifiedTextReflowKey {
    pub const fn digest(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone)]
pub struct UiQualifiedTextLayoutRequest {
    identity: UiQualifiedTextLayoutRequestIdentity,
    reflow_key: UiQualifiedTextReflowKey,
    input: UiTextParagraphAdmissionInput,
    fonts: Arc<crate::UiGlobalFontCollection>,
}

impl UiQualifiedTextLayoutRequest {
    pub fn new(
        input: UiTextParagraphAdmissionInput,
        fonts: Arc<crate::UiGlobalFontCollection>,
    ) -> Self {
        let parts = UiQualifiedTextRequestParts::of_input(&input, fonts.identity_digest());
        let identity = parts.identity_at(input.constraints.width_millipoints());
        let reflow_key = parts.reflow_key();
        Self {
            identity,
            reflow_key,
            input,
            fonts,
        }
    }

    pub const fn identity(&self) -> UiQualifiedTextLayoutRequestIdentity {
        self.identity
    }

    /// The key this request shares with itself at every other width.
    pub const fn reflow_key(&self) -> UiQualifiedTextReflowKey {
        self.reflow_key
    }

    /// The width this request fits its lines to.
    pub const fn width_millipoints(&self) -> u32 {
        self.input.constraints.width_millipoints()
    }

    pub fn qualify(self) -> Result<crate::UiQualifiedTextLayout, crate::UiTextQualificationDenial> {
        crate::qualification::qualify_request(self)
    }

    pub(crate) fn fonts(&self) -> &Arc<crate::UiGlobalFontCollection> {
        &self.fonts
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiTextParagraphAdmissionInput,
        Arc<crate::UiGlobalFontCollection>,
    ) {
        (self.input, self.fonts)
    }
}

/// Everything a request names except its width, borrowed from the request or
/// from the paragraph it admitted.
pub(crate) struct UiQualifiedTextRequestParts<'a> {
    collection_identity: [u8; 32],
    source: &'a str,
    constraints: &'a crate::UiTextParagraphConstraints,
    profile_generation: UiTextProfileGeneration,
    font_collection_generation: UiFontCollectionGeneration,
    text_scale_generation: UiTextScaleGeneration,
    styles: &'a [crate::UiTextStyleSpan],
}

impl<'a> UiQualifiedTextRequestParts<'a> {
    fn of_input(input: &'a UiTextParagraphAdmissionInput, collection_identity: [u8; 32]) -> Self {
        Self {
            collection_identity,
            source: &input.source,
            constraints: &input.constraints,
            profile_generation: input.profile_generation,
            font_collection_generation: input.font_collection_generation,
            text_scale_generation: input.text_scale_generation,
            styles: &input.styles,
        }
    }

    /// The parts of the request `shaped` was admitted for from `fonts`.
    pub(crate) fn of_shaped(
        shaped: &'a crate::UiShapedTextParagraph,
        fonts: &crate::UiGlobalFontCollection,
    ) -> Self {
        Self {
            collection_identity: fonts.identity_digest(),
            source: shaped.source(),
            constraints: shaped.constraints(),
            profile_generation: shaped.profile_generation(),
            font_collection_generation: shaped.font_collection_generation(),
            text_scale_generation: shaped.text_scale_generation(),
            styles: shaped.styles(),
        }
    }

    /// The identity of this request fitting its lines to `width_millipoints`.
    pub(crate) fn identity_at(
        &self,
        width_millipoints: u32,
    ) -> UiQualifiedTextLayoutRequestIdentity {
        UiQualifiedTextLayoutRequestIdentity::from_text_mechanics(self.digest(
            b"worth-ui-qualified-text-layout-request-v1\0",
            Some(width_millipoints),
        ))
    }

    pub(crate) fn reflow_key(&self) -> UiQualifiedTextReflowKey {
        UiQualifiedTextReflowKey(self.digest(b"worth-ui-qualified-text-reflow-v1\0", None))
    }

    fn digest(&self, domain: &[u8], width_millipoints: Option<u32>) -> [u8; 32] {
        let constraints = self.constraints;
        let mut hash = Sha256::new();
        hash.update(domain);
        hash.update(self.collection_identity);
        hash_bytes(&mut hash, self.source.as_bytes());
        hash_bytes(&mut hash, constraints.language().as_bytes());
        hash.update([direction_rank(constraints.base_direction())]);
        hash.update([wrap_rank(constraints.wrap())]);
        hash.update([alignment_rank(constraints.alignment())]);
        hash.update([overflow_rank(constraints.overflow())]);
        hash.update(constraints.font_size_millipoints().to_le_bytes());
        if let Some(width_millipoints) = width_millipoints {
            hash.update(width_millipoints.to_le_bytes());
        }
        hash.update(constraints.line_height_millipoints().to_le_bytes());
        hash.update(constraints.letter_spacing_millipoints().to_le_bytes());
        hash.update(constraints.word_spacing_millipoints().to_le_bytes());
        hash.update(constraints.tab_interval_millipoints().to_le_bytes());
        hash.update(constraints.maximum_lines().to_le_bytes());
        hash.update(self.profile_generation.get().to_le_bytes());
        hash.update(self.font_collection_generation.get().to_le_bytes());
        hash.update(self.text_scale_generation.get().to_le_bytes());
        hash.update(
            u64::try_from(self.styles.len())
                .expect("qualified style capacity fits u64")
                .to_le_bytes(),
        );
        for span in self.styles {
            hash.update(span.original_range().start().to_le_bytes());
            hash.update(span.original_range().end().to_le_bytes());
            hash.update(span.style().identity_digest());
        }
        hash.finalize().into()
    }
}

fn hash_bytes(hash: &mut Sha256, bytes: &[u8]) {
    hash.update(
        u64::try_from(bytes.len())
            .expect("qualified text capacity fits u64")
            .to_le_bytes(),
    );
    hash.update(bytes);
}

const fn direction_rank(value: crate::UiTextBaseDirection) -> u8 {
    match value {
        crate::UiTextBaseDirection::Auto => 0,
        crate::UiTextBaseDirection::LeftToRight => 1,
        crate::UiTextBaseDirection::RightToLeft => 2,
    }
}

const fn wrap_rank(value: crate::UiTextWrap) -> u8 {
    match value {
        crate::UiTextWrap::None => 0,
        crate::UiTextWrap::UnicodeWord => 1,
        crate::UiTextWrap::Grapheme => 2,
    }
}

const fn alignment_rank(value: crate::UiTextAlignment) -> u8 {
    match value {
        crate::UiTextAlignment::Start => 0,
        crate::UiTextAlignment::Center => 1,
        crate::UiTextAlignment::End => 2,
    }
}

const fn overflow_rank(value: crate::UiTextOverflow) -> u8 {
    match value {
        crate::UiTextOverflow::Clip => 0,
        crate::UiTextOverflow::Ellipsis => 1,
    }
}
