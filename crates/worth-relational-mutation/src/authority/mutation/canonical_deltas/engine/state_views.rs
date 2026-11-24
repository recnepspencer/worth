use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;

#[derive(Debug, Clone, Copy)]
pub(in crate::internal_port::authority::mutation) struct RelationState<'a> {
    pub(in crate::internal_port::authority::mutation) source: Option<EntityId>,
    pub(in crate::internal_port::authority::mutation) target: Option<EntityId>,
    pub(in crate::internal_port::authority::mutation) authoritative_state:
        Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::internal_port::authority::mutation) struct EntityAuthoritativeState<'a> {
    pub(in crate::internal_port::authority::mutation) authoritative_state:
        Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
}
