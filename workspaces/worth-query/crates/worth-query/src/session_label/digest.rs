use super::namespace::WorthQuerySessionNamespace;
use super::segment::WorthQuerySessionLabelSegment;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

pub(crate) fn derive_session_label_identity(
    namespace: &WorthQuerySessionNamespace,
    name_segments: &[WorthQuerySessionLabelSegment],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::SessionLabelIdentity)
        .field_value(
            WorthQueryEvidenceTag::new("session_label_namespace"),
            namespace.as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("session_label_name_segment_count"),
            name_segments.len(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("session_label_name_segments"),
            name_segments
                .iter()
                .map(WorthQuerySessionLabelSegment::as_str),
        )
        .seal()
}
