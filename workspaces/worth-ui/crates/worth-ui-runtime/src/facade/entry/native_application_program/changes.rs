use super::UiNativeApplicationProgramDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeComponentPresenceChange {
    authored_semantic_identity: Box<str>,
    present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeComponentSemanticTextChange {
    pub(super) authored_semantic_identity: Box<str>,
    text: Box<str>,
    pub(super) expected_revision: u64,
    spans: Option<Box<[crate::facade::registry::descriptor::ComponentSemanticTextSpanContract]>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeThemeTokenValueChange {
    pub(super) token: crate::facade::registry::descriptor::ThemeTokenId,
    value: crate::facade::registry::descriptor::ThemeTokenValue,
    pub(super) expected_revision: u64,
}

impl UiNativeComponentPresenceChange {
    pub fn new(
        authored_semantic_identity: impl Into<Box<str>>,
        present: bool,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let identity = authored_semantic_identity.into();
        if !identity.starts_with("component:") || identity.len() == "component:".len() {
            return Err(UiNativeApplicationProgramDenial::InvalidComponentIdentity);
        }
        Ok(Self {
            authored_semantic_identity: identity,
            present,
        })
    }

    pub(crate) fn authored_semantic_identity(&self) -> &str {
        &self.authored_semantic_identity
    }

    pub(crate) const fn present(&self) -> bool {
        self.present
    }
}

impl UiNativeComponentSemanticTextChange {
    pub fn new(
        authored_semantic_identity: impl Into<Box<str>>,
        text: impl Into<Box<str>>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        Self::successor(authored_semantic_identity, 0, text)
    }

    /// Build an application-owned semantic-text successor against the exact
    /// revision last admitted for this authored component.
    /// Empty text clears the value while retaining its declared formatting.
    pub fn successor(
        authored_semantic_identity: impl Into<Box<str>>,
        expected_revision: u64,
        text: impl Into<Box<str>>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let identity = authored_semantic_identity.into();
        let text = text.into();
        if !identity.starts_with("component:") || identity.len() == "component:".len() {
            return Err(UiNativeApplicationProgramDenial::InvalidComponentIdentity);
        }
        Ok(Self {
            authored_semantic_identity: identity,
            text,
            expected_revision,
            spans: None,
        })
    }

    pub(crate) fn authored_semantic_identity(&self) -> &str {
        &self.authored_semantic_identity
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    pub fn with_spans(
        mut self,
        spans: impl IntoIterator<
            Item = crate::facade::registry::descriptor::ComponentSemanticTextSpanContract,
        >,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let spans = spans.into_iter().collect::<Vec<_>>();
        if spans.is_empty()
            || spans.len() > worth_ui_text::UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH
        {
            return Err(UiNativeApplicationProgramDenial::InvalidSemanticTextSpans);
        }
        self.spans = Some(spans.into_boxed_slice());
        Ok(self)
    }

    pub(crate) fn spans(
        &self,
    ) -> Option<&[crate::facade::registry::descriptor::ComponentSemanticTextSpanContract]> {
        self.spans.as_deref()
    }
}

impl UiNativeThemeTokenValueChange {
    pub fn new(
        token: crate::facade::registry::descriptor::ThemeTokenId,
        value: crate::facade::registry::descriptor::ThemeTokenValue,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        Self::successor(token, 0, value)
    }

    pub fn successor(
        token: crate::facade::registry::descriptor::ThemeTokenId,
        expected_revision: u64,
        value: crate::facade::registry::descriptor::ThemeTokenValue,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        if !value.is_valid() {
            return Err(UiNativeApplicationProgramDenial::InvalidThemeTokenValue);
        }
        Ok(Self {
            token,
            value,
            expected_revision,
        })
    }

    pub(crate) const fn token(&self) -> &crate::facade::registry::descriptor::ThemeTokenId {
        &self.token
    }

    pub(crate) const fn value(&self) -> &crate::facade::registry::descriptor::ThemeTokenValue {
        &self.value
    }

    pub(crate) const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}
