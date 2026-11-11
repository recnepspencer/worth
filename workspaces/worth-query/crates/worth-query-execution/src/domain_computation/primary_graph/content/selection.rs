//! Erased typed selector facts; all reader mechanics compile once per owner.
use super::super::value_decode::ValueDecodeSession;
use worth_foundational::facade::{AspectValue, ContentRefId};
use worth_query_declaration::facade::application_schema::{
    fragment::{ColumnIdentity, RecordSelection},
    ApplicationContentBinding, ApplicationContentReferenceBinding,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding as Installed;

pub(in crate::domain_computation::primary_graph) struct ContentFieldSelection {
    pub column: ColumnIdentity,
    pub matches: fn(&Installed) -> bool,
    pub decode: DecodeReference,
}
pub(in crate::domain_computation::primary_graph) type DecodeReference =
    fn(&ValueDecodeSession<'_>, &str, &str, &str, &AspectValue) -> Option<ContentRefId>;
impl ContentFieldSelection {
    pub(in crate::domain_computation::primary_graph) fn of<F, C>(field: F) -> Self
    where
        F: RecordSelection,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        Self {
            column: field.identity(),
            matches: Installed::matches_content::<C>,
            decode: decode::<F::Binding>,
        }
    }
}
fn decode<B: ApplicationContentReferenceBinding>(
    session: &ValueDecodeSession<'_>,
    entity: &str,
    aspect: &str,
    field: &str,
    raw: &AspectValue,
) -> Option<ContentRefId> {
    session
        .decode::<B>(entity, aspect, field, raw)
        .as_ref()
        .map(B::reference)
}
