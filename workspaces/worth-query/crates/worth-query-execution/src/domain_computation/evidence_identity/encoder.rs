use std::sync::Arc;

use super::artifact::WorthQueryEvidenceIdentity;
use super::foundational::{derive_evidence_identity, digest_entry, text_entry};
use super::scheme::WorthQueryEvidenceIdentityScheme;
use super::scope::WorthQueryEvidenceScope;
use super::tag::WorthQueryEvidenceTag;
use worth_foundational::facade::{
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalDigestId, FoundationalIdentityKind,
};
use worth_runtime_bridge::facade::{BridgeIdentityEvidence, BridgeTruthBoundaryBridgedIdentity};

pub fn worth_query_evidence_identity(
    scope: WorthQueryEvidenceScope,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentityEncoder::new(scope, WorthQueryEvidenceIdentityScheme::V1)
}

#[cfg(test)]
pub(crate) fn worth_query_evidence_identity_with_scheme(
    scope: WorthQueryEvidenceScope,
    scheme: WorthQueryEvidenceIdentityScheme,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentityEncoder::new(scope, scheme)
}

pub struct WorthQueryEvidenceIdentityEncoder {
    scope: WorthQueryEvidenceScope,
    scheme: WorthQueryEvidenceIdentityScheme,
    entries: Vec<CanonicalBasisEntry>,
}

impl WorthQueryEvidenceIdentityEncoder {
    pub(crate) fn new(
        scope: WorthQueryEvidenceScope,
        scheme: WorthQueryEvidenceIdentityScheme,
    ) -> Self {
        let entries = vec![
            text_entry(
                "evidence.scheme",
                CanonicalBasisEntryKind::Header,
                scheme.as_str(),
            ),
            text_entry(
                "evidence.scope",
                CanonicalBasisEntryKind::Header,
                scope.as_str(),
            ),
        ];
        Self {
            scope,
            scheme,
            entries,
        }
    }

    pub fn field_shape(mut self, tag: WorthQueryEvidenceTag, value: impl AsRef<str>) -> Self {
        self.push_text(CanonicalBasisEntryKind::Shape, tag, value);
        self
    }

    pub fn field_evidence_identity(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: &WorthQueryEvidenceIdentity,
    ) -> Self {
        self.push_text(CanonicalBasisEntryKind::Identity, tag, value.as_str());
        self
    }

    pub fn field_digest(mut self, tag: WorthQueryEvidenceTag, value: &CanonicalDigestId) -> Self {
        self.entries.push(digest_entry(
            field_locus(tag),
            CanonicalBasisEntryKind::Identity,
            value,
        ));
        self
    }

    pub fn field_bridge_authority_identity<Kind>(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: &BridgeTruthBoundaryBridgedIdentity<Arc<str>, Kind>,
    ) -> Self
    where
        Kind: FoundationalIdentityKind,
    {
        self.push_text(
            CanonicalBasisEntryKind::Identity,
            tag,
            value.value().as_ref(),
        );
        self
    }

    pub fn field_bridge_retained_evidence_identity(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: &BridgeIdentityEvidence,
    ) -> Self {
        self.push_text(
            CanonicalBasisEntryKind::Identity,
            tag,
            worth_runtime_bridge::facade::bridge_identity_reporting_label(value),
        );
        self
    }

    pub fn field_value(mut self, tag: WorthQueryEvidenceTag, value: impl AsRef<str>) -> Self {
        self.push_text(CanonicalBasisEntryKind::Value, tag, value);
        self
    }

    pub fn field_bool(mut self, tag: WorthQueryEvidenceTag, value: bool) -> Self {
        self.push_text(
            CanonicalBasisEntryKind::Value,
            tag,
            if value { "true" } else { "false" },
        );
        self
    }

    pub fn field_usize(mut self, tag: WorthQueryEvidenceTag, value: usize) -> Self {
        self.push_text(CanonicalBasisEntryKind::Value, tag, value.to_string());
        self
    }

    pub fn field_evidence_identity_sequence<'a, I>(
        mut self,
        tag: WorthQueryEvidenceTag,
        values: I,
    ) -> Self
    where
        I: IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
    {
        let mut count = 0usize;
        for (index, value) in values.into_iter().enumerate() {
            self.entries.push(text_entry(
                sequence_item_locus(tag, index),
                CanonicalBasisEntryKind::Identity,
                value.as_str(),
            ));
            count = index + 1;
        }
        self.push_sequence_count(tag, count);
        self
    }

    pub fn field_value_sequence<I, S>(mut self, tag: WorthQueryEvidenceTag, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut count = 0usize;
        for (index, value) in values.into_iter().enumerate() {
            self.entries.push(text_entry(
                sequence_item_locus(tag, index),
                CanonicalBasisEntryKind::Value,
                value.as_ref(),
            ));
            count = index + 1;
        }
        self.push_sequence_count(tag, count);
        self
    }

    pub fn optional_shape(self, tag: WorthQueryEvidenceTag, value: Option<&str>) -> Self {
        match value {
            Some(value) => self.field_shape(tag, value),
            None => self,
        }
    }
    pub fn optional_evidence_identity(
        self,
        tag: WorthQueryEvidenceTag,
        value: Option<&WorthQueryEvidenceIdentity>,
    ) -> Self {
        match value {
            Some(value) => self.field_evidence_identity(tag, value),
            None => self,
        }
    }

    pub fn optional_identity(
        self,
        tag: WorthQueryEvidenceTag,
        value: Option<impl AsRef<str>>,
    ) -> Self {
        match value.as_ref().map(|value| value.as_ref()) {
            Some(value) => self.field_value(tag, value),
            None => self,
        }
    }

    pub fn optional_value(self, tag: WorthQueryEvidenceTag, value: Option<&str>) -> Self {
        match value {
            Some(value) => self.field_value(tag, value),
            None => self,
        }
    }

    pub fn seal(self) -> WorthQueryEvidenceIdentity {
        WorthQueryEvidenceIdentity::new(derive_evidence_identity(
            self.scope,
            self.scheme,
            self.entries,
        ))
    }

    fn push_text(
        &mut self,
        kind: CanonicalBasisEntryKind,
        tag: WorthQueryEvidenceTag,
        value: impl AsRef<str>,
    ) {
        self.entries
            .push(text_entry(field_locus(tag), kind, value.as_ref()));
    }

    fn push_sequence_count(&mut self, tag: WorthQueryEvidenceTag, count: usize) {
        self.entries.push(text_entry(
            sequence_count_locus(tag),
            CanonicalBasisEntryKind::Shape,
            count.to_string(),
        ));
    }
}

fn field_locus(tag: WorthQueryEvidenceTag) -> String {
    format!("evidence.field.{}", tag.as_str())
}

fn sequence_count_locus(tag: WorthQueryEvidenceTag) -> String {
    format!("evidence.sequence.{}.count", tag.as_str())
}

fn sequence_item_locus(tag: WorthQueryEvidenceTag, index: usize) -> String {
    format!("evidence.sequence.{}.item.{index}", tag.as_str())
}
