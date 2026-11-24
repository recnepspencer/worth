use crate::identity::data::EntityId;

#[derive(Debug, Clone, Copy)]
pub(in crate::authority::mutation) struct RelationState<'a> {
    pub(in crate::authority::mutation) source: Option<EntityId>,
    pub(in crate::authority::mutation) target: Option<EntityId>,
    pub(in crate::authority::mutation) authoritative_state:
        Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::authority::mutation) struct EntityAuthoritativeState<'a> {
    pub(in crate::authority::mutation) authoritative_state:
        Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
}
