//! Entity metadata and extra fields in the original portable image order.
use super::{arena::WireKind, aspects::State, *};
use crate::storage::substrate::{EntityExtra, EntityRecordKind, VersionedEntityMetadata};
impl WireKind for EntityRecordKind {
    fn plan(
        schema: &RelationalBranchRootSchemaAuthority,
        kind: crate::identity::data::KindId,
    ) -> bool {
        schema.aspect_plans().entity_plans.contains_key(&kind)
    }
    fn meta_kind(meta: &Self::Meta) -> crate::identity::data::KindId {
        meta.kind_id
    }
    fn meta_state(
        meta: &Self::Meta,
    ) -> Option<&worth_foundational::facade::AuthoritativeRecordAspectState> {
        meta.authoritative_aspect_state()
    }
    fn extra_state(
        extra: &Self::Extra,
    ) -> Option<&worth_foundational::facade::AuthoritativeRecordAspectState> {
        extra.authoritative_aspect_state()
    }
    fn serialize_meta<S: serde::Serializer>(
        meta: &VersionedEntityMetadata,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("VersionedEntityMetadataImage", 6)?;
        state.serialize_field("effective_at", &meta.effective_at)?;
        state.serialize_field("retired_at", &meta.retired_at)?;
        state.serialize_field("generation", &meta.generation)?;
        state.serialize_field("kind_id", &meta.kind_id)?;
        state.serialize_field("lineage_id", &meta.lineage_id)?;
        state.serialize_field(
            "authoritative_aspect_state",
            &meta.authoritative_aspect_state().map(State),
        )?;
        state.end()
    }
    fn serialize_extra<S: serde::Serializer>(
        extra: &EntityExtra,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EntityExtraImage", 3)?;
        state.serialize_field("structural_fingerprint", &extra.structural_fingerprint)?;
        state.serialize_field("lineage_id", &extra.lineage_id)?;
        state.serialize_field(
            "authoritative_aspect_state",
            &extra.authoritative_aspect_state().map(State),
        )?;
        state.end()
    }
}
