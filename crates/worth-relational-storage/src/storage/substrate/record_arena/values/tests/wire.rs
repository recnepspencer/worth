use super::*;
#[derive(serde::Serialize)]
pub(super) struct LegacyEntity<S> {
    pub(super) structural_fingerprint: Option<crate::identity::data::StructuralFingerprint>,
    pub(super) lineage_id: Option<crate::identity::data::LineageId>,
    pub(super) authoritative_aspect_state: Option<S>,
}
#[derive(serde::Serialize)]
pub(super) struct LegacyMeta<S> {
    pub(super) effective_at: VersionId,
    pub(super) retired_at: Option<VersionId>,
    pub(super) generation: u32,
    pub(super) kind_id: KindId,
    pub(super) lineage_id: Option<crate::identity::data::LineageId>,
    pub(super) authoritative_aspect_state: Option<S>,
}
#[derive(serde::Serialize)]
pub(super) struct LegacyRelation<E, S> {
    pub(super) endpoints: Option<E>,
    pub(super) authoritative_aspect_state: Option<S>,
}
#[derive(serde::Serialize)]
pub(super) struct LegacyRelationMeta<E, S> {
    pub(super) effective_at: VersionId,
    pub(super) retired_at: Option<VersionId>,
    pub(super) generation: u32,
    pub(super) kind_id: KindId,
    pub(super) endpoints: E,
    pub(super) authoritative_aspect_state: Option<S>,
}
pub(super) fn equal(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert_eq!(
        rmp_serde::to_vec_named(a).unwrap(),
        rmp_serde::to_vec_named(b).unwrap()
    );
    assert_eq!(rmp_serde::to_vec(a).unwrap(), rmp_serde::to_vec(b).unwrap());
    assert_eq!(
        serde_json::to_vec(a).unwrap(),
        serde_json::to_vec(b).unwrap()
    );
}
