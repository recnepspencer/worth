use super::digest::derive_session_label_identity;
use super::error::WorthQuerySessionLabelError;
use super::namespace::WorthQuerySessionNamespace;
use super::sealed::SealedWorthQuerySessionLabel;
use super::segment::WorthQuerySessionLabelSegment;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope};

#[derive(Clone, Debug)]
pub struct WorthQuerySessionLabel {
    namespace: WorthQuerySessionNamespace,
    name_segments: Vec<WorthQuerySessionLabelSegment>,
    display: String,
    identity_digest: WorthQueryEvidenceIdentity,
}

impl PartialEq for WorthQuerySessionLabel {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.name_segments == other.name_segments
    }
}

impl Eq for WorthQuerySessionLabel {}

impl PartialOrd for WorthQuerySessionLabel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorthQuerySessionLabel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.namespace
            .cmp(&other.namespace)
            .then_with(|| self.name_segments.cmp(&other.name_segments))
    }
}

impl std::hash::Hash for WorthQuerySessionLabel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.name_segments.hash(state);
    }
}

impl WorthQuerySessionLabel {
    pub fn scoped(
        namespace: WorthQuerySessionNamespace,
        name_segments: impl IntoIterator<Item = WorthQuerySessionLabelSegment>,
    ) -> Result<Self, WorthQuerySessionLabelError> {
        let name_segments = name_segments.into_iter().collect::<Vec<_>>();
        if name_segments.is_empty() {
            return Err(WorthQuerySessionLabelError::MissingNameSegments);
        }
        let display = render_display(&namespace, &name_segments);
        let identity_digest = derive_session_label_identity(&namespace, &name_segments)?;
        Ok(Self::new(SealedWorthQuerySessionLabel::new(
            namespace,
            name_segments,
            display,
            identity_digest,
        )))
    }

    pub fn scoped_strs<I, S>(
        namespace: impl Into<String>,
        name_segments: I,
    ) -> Result<Self, WorthQuerySessionLabelError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::scoped(
            WorthQuerySessionNamespace::new(namespace)?,
            name_segments
                .into_iter()
                .map(WorthQuerySessionLabelSegment::new)
                .collect::<Result<Vec<_>, _>>()?,
        )
    }

    pub fn namespace(&self) -> &WorthQuerySessionNamespace {
        &self.namespace
    }

    pub fn name_segments(&self) -> &[WorthQuerySessionLabelSegment] {
        &self.name_segments
    }

    pub fn identity_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity_digest
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn identity_scope() -> WorthQueryEvidenceScope {
        WorthQueryEvidenceScope::SessionLabelIdentity
    }

    pub(crate) fn new(sealed: SealedWorthQuerySessionLabel) -> Self {
        Self {
            namespace: sealed.namespace,
            name_segments: sealed.name_segments,
            display: sealed.display,
            identity_digest: sealed.identity_digest,
        }
    }
}

impl std::fmt::Display for WorthQuerySessionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display())
    }
}

fn render_display(
    namespace: &WorthQuerySessionNamespace,
    name_segments: &[WorthQuerySessionLabelSegment],
) -> String {
    let mut parts = Vec::with_capacity(name_segments.len() + 1);
    parts.push(namespace.as_str().to_string());
    parts.extend(
        name_segments
            .iter()
            .map(WorthQuerySessionLabelSegment::as_str)
            .map(str::to_string),
    );
    parts.join(".")
}
