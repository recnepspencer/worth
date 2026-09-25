use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, FieldKey,
    StructAspectValue,
};

use super::contracts::ProjectionAspectScope;

pub trait EntityRecordProjection: Sized {
    const KIND: KindId;

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::empty()
    }

    fn from_record(record: EntityProjectionRecord<'_>) -> Option<Self>;
}

pub trait RelationRecordProjection: Sized {
    const KIND: KindId;

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::empty()
    }

    fn from_record(record: RelationProjectionRecord<'_>) -> Option<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct EntityProjectionRecord<'a> {
    entity_id: EntityId,
    kind_id: KindId,
    kind_name: &'a str,
    lifecycle: RecordLifecycleState,
    created_at_version: VersionId,
    authoritative_aspect_state: Option<&'a AuthoritativeRecordAspectState>,
    projection_scope: &'a ProjectionAspectScope,
}

impl<'a> EntityProjectionRecord<'a> {
    pub(crate) fn new(
        record: &'a EntityReadRecord,
        projection_scope: &'a ProjectionAspectScope,
    ) -> Self {
        Self {
            entity_id: record.entity_id,
            kind_id: record.kind.kind_id,
            kind_name: &record.kind.kind_name,
            lifecycle: record.lifecycle,
            created_at_version: record.created_at_version,
            authoritative_aspect_state: record.authoritative_aspect_state.as_ref(),
            projection_scope,
        }
    }

    pub(super) fn from_slot(
        entity_id: EntityId,
        slot: &'a crate::storage::substrate::SlotView<
            '_,
            crate::storage::substrate::EntityRecordKind,
        >,
        kind: &'a crate::schema::data::KindResolution,
        created_at_version: VersionId,
        projection_scope: &'a ProjectionAspectScope,
    ) -> Self {
        Self {
            entity_id,
            kind_id: kind.kind_id,
            kind_name: &kind.kind_name,
            lifecycle: slot.lifecycle(),
            created_at_version,
            authoritative_aspect_state: slot.extra().authoritative_aspect_state.as_ref(),
            projection_scope,
        }
    }

    pub const fn entity_id(self) -> EntityId {
        self.entity_id
    }

    pub const fn kind_id(self) -> KindId {
        self.kind_id
    }

    pub fn kind_name(self) -> &'a str {
        self.kind_name
    }

    pub const fn lifecycle(self) -> RecordLifecycleState {
        self.lifecycle
    }

    pub const fn created_at_version(self) -> VersionId {
        self.created_at_version
    }

    fn authoritative_aspect_state(self) -> Option<&'a AuthoritativeRecordAspectState> {
        self.authoritative_aspect_state
    }

    pub fn aspect_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
    ) -> Option<&'a AspectValue> {
        if !self.projection_scope.contains_whole_aspect(aspect_key) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(value) => Some(value),
            ContractValidatedAspectValueView::Struct(_) => None,
        }
    }

    pub fn struct_aspect_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
    ) -> Option<&'a StructAspectValue> {
        if !self.projection_scope.contains_whole_aspect(aspect_key) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(_) => None,
            ContractValidatedAspectValueView::Struct(value) => Some(value),
        }
    }

    pub fn aspect_field_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
        field: &FieldKey,
    ) -> Option<&'a AspectValue> {
        if !self.projection_scope.contains_field(aspect_key, field) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(_) => None,
            ContractValidatedAspectValueView::Struct(value) => value.get(field),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RelationProjectionRecord<'a> {
    record: &'a RelationReadRecord,
    projection_scope: &'a ProjectionAspectScope,
}

impl<'a> RelationProjectionRecord<'a> {
    pub(crate) const fn new(
        record: &'a RelationReadRecord,
        projection_scope: &'a ProjectionAspectScope,
    ) -> Self {
        Self {
            record,
            projection_scope,
        }
    }

    pub const fn relation_id(self) -> RelationId {
        self.record.relation_id
    }

    pub const fn kind_id(self) -> KindId {
        self.record.kind.kind_id
    }

    pub fn kind_name(self) -> &'a str {
        &self.record.kind.kind_name
    }

    pub const fn source(self) -> EntityId {
        self.record.source
    }

    pub const fn target(self) -> EntityId {
        self.record.target
    }

    pub const fn lifecycle(self) -> RecordLifecycleState {
        self.record.lifecycle
    }

    fn authoritative_aspect_state(self) -> Option<&'a AuthoritativeRecordAspectState> {
        self.record.authoritative_aspect_state.as_ref()
    }

    pub fn aspect_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
    ) -> Option<&'a AspectValue> {
        if !self.projection_scope.contains_whole_aspect(aspect_key) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(value) => Some(value),
            ContractValidatedAspectValueView::Struct(_) => None,
        }
    }

    pub fn struct_aspect_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
    ) -> Option<&'a StructAspectValue> {
        if !self.projection_scope.contains_whole_aspect(aspect_key) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(_) => None,
            ContractValidatedAspectValueView::Struct(value) => Some(value),
        }
    }

    pub fn aspect_field_value(
        self,
        aspect_key: &worth_foundational::facade::AspectKey,
        field: &FieldKey,
    ) -> Option<&'a AspectValue> {
        if !self.projection_scope.contains_field(aspect_key, field) {
            return None;
        }
        match self.authoritative_aspect_state()?.get(aspect_key)?.view() {
            ContractValidatedAspectValueView::Scalar(_) => None,
            ContractValidatedAspectValueView::Struct(value) => value.get(field),
        }
    }
}
