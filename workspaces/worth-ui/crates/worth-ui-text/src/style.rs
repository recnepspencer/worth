use std::sync::Arc;

use worth_ui_host_contract::UiTextOriginalRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiOpenTypeFeature {
    tag: [u8; 4],
    value: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiFontVariationCoordinate {
    axis: [u8; 4],
    value_milli: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiTextStyle {
    language: Arc<str>,
    font_size_millipoints: u32,
    letter_spacing_millipoints: i32,
    word_spacing_millipoints: i32,
    family_stack: crate::UiFontFamilyStack,
    face_request: crate::UiTextFaceRequest,
    features: Box<[UiOpenTypeFeature]>,
    variations: Box<[UiFontVariationCoordinate]>,
}

pub struct UiTextStyleInput {
    pub language: Arc<str>,
    pub font_size_millipoints: u32,
    pub letter_spacing_millipoints: i32,
    pub word_spacing_millipoints: i32,
    pub family_stack: crate::UiFontFamilyStack,
    pub face_request: crate::UiTextFaceRequest,
    pub features: Box<[UiOpenTypeFeature]>,
    pub variations: Box<[UiFontVariationCoordinate]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiTextStyleSpan {
    original_range: UiTextOriginalRange,
    style: UiTextStyle,
}

impl UiOpenTypeFeature {
    pub fn new(tag: [u8; 4], value: u32) -> Option<Self> {
        qualified_tag(tag).then_some(Self { tag, value })
    }

    pub const fn tag(self) -> [u8; 4] {
        self.tag
    }

    pub const fn value(self) -> u32 {
        self.value
    }
}

impl UiFontVariationCoordinate {
    pub fn new(axis: [u8; 4], value_milli: i32) -> Option<Self> {
        qualified_tag(axis).then_some(Self { axis, value_milli })
    }

    pub const fn axis(self) -> [u8; 4] {
        self.axis
    }

    pub const fn value_milli(self) -> i32 {
        self.value_milli
    }
}

impl UiTextStyle {
    pub fn new(mut input: UiTextStyleInput) -> Option<Self> {
        let language = crate::language::admit_language(&input.language)?;
        if input.font_size_millipoints == 0 {
            return None;
        }
        input.features.sort_by_key(|feature| feature.tag());
        input.variations.sort_by_key(|variation| variation.axis());
        if input
            .features
            .windows(2)
            .any(|pair| pair[0].tag() == pair[1].tag())
            || input
                .variations
                .windows(2)
                .any(|pair| pair[0].axis() == pair[1].axis())
        {
            return None;
        }
        Some(Self {
            language,
            font_size_millipoints: input.font_size_millipoints,
            letter_spacing_millipoints: input.letter_spacing_millipoints,
            word_spacing_millipoints: input.word_spacing_millipoints,
            family_stack: input.family_stack,
            face_request: input.face_request,
            features: input.features,
            variations: input.variations,
        })
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub const fn font_size_millipoints(&self) -> u32 {
        self.font_size_millipoints
    }

    pub const fn letter_spacing_millipoints(&self) -> i32 {
        self.letter_spacing_millipoints
    }

    pub const fn word_spacing_millipoints(&self) -> i32 {
        self.word_spacing_millipoints
    }

    pub const fn family_stack(&self) -> &crate::UiFontFamilyStack {
        &self.family_stack
    }

    pub const fn face_request(&self) -> crate::UiTextFaceRequest {
        self.face_request
    }

    pub fn features(&self) -> &[UiOpenTypeFeature] {
        &self.features
    }

    pub fn variations(&self) -> &[UiFontVariationCoordinate] {
        &self.variations
    }

    pub fn identity_digest(&self) -> [u8; 32] {
        use sha2::Digest;
        use worth_ui_host_contract::UiCountedSha256;
        let mut digest = UiCountedSha256::new();
        digest.update(b"worth-ui-text-style-v3\0");
        hash_identity_bytes(&mut digest, b"language", self.language.as_bytes());
        digest.update(self.font_size_millipoints.to_le_bytes());
        digest.update(self.letter_spacing_millipoints.to_le_bytes());
        digest.update(self.word_spacing_millipoints.to_le_bytes());
        hash_identity_len(&mut digest, b"families", self.family_stack.families().len());
        for family in self.family_stack.families() {
            digest.update(family.digest());
        }
        digest.update(self.face_request.weight().to_le_bytes());
        digest.update(self.face_request.width_milli_percent().to_le_bytes());
        digest.update([match self.face_request.slant() {
            worth_ui_host_contract::UiFontSlant::Upright => 0,
            worth_ui_host_contract::UiFontSlant::Italic => 1,
            worth_ui_host_contract::UiFontSlant::Oblique => 2,
        }]);
        hash_identity_len(&mut digest, b"features", self.features.len());
        for feature in &self.features {
            digest.update(feature.tag());
            digest.update(feature.value().to_le_bytes());
        }
        hash_identity_len(&mut digest, b"variations", self.variations.len());
        for variation in &self.variations {
            digest.update(variation.axis());
            digest.update(variation.value_milli().to_le_bytes());
        }
        digest.finalize().into()
    }

    pub fn from_paragraph_constraints(constraints: &crate::UiTextParagraphConstraints) -> Self {
        Self {
            language: Arc::from(constraints.language()),
            font_size_millipoints: constraints.font_size_millipoints(),
            letter_spacing_millipoints: constraints.letter_spacing_millipoints(),
            word_spacing_millipoints: constraints.word_spacing_millipoints(),
            family_stack: crate::UiFontFamilyStack::profile_sans(),
            face_request: crate::UiTextFaceRequest::regular(),
            features: Box::new([]),
            variations: Box::new([]),
        }
    }
}

impl UiTextStyleSpan {
    pub fn new(original_range: UiTextOriginalRange, style: UiTextStyle) -> Option<Self> {
        (!original_range.is_empty()).then_some(Self {
            original_range,
            style,
        })
    }

    pub const fn original_range(&self) -> UiTextOriginalRange {
        self.original_range
    }

    pub const fn style(&self) -> &UiTextStyle {
        &self.style
    }

    pub fn whole_paragraph(
        source: &str,
        constraints: &crate::UiTextParagraphConstraints,
    ) -> Option<Self> {
        let end = u32::try_from(source.len()).ok()?;
        Self::new(
            UiTextOriginalRange::from_text_mechanics(0, end)?,
            UiTextStyle::from_paragraph_constraints(constraints),
        )
    }
}

fn qualified_tag(tag: [u8; 4]) -> bool {
    tag.iter().all(|byte| (0x20..=0x7e).contains(byte))
}

fn hash_identity_bytes(
    digest: &mut worth_ui_host_contract::UiCountedSha256,
    domain: &[u8],
    bytes: &[u8],
) {
    use sha2::Digest;
    hash_identity_len(digest, domain, bytes.len());
    digest.update(bytes);
}

fn hash_identity_len(
    digest: &mut worth_ui_host_contract::UiCountedSha256,
    domain: &[u8],
    len: usize,
) {
    use sha2::Digest;
    digest.update(domain);
    digest.update(
        u64::try_from(len)
            .expect("qualified style identity length fits u64")
            .to_le_bytes(),
    );
}
