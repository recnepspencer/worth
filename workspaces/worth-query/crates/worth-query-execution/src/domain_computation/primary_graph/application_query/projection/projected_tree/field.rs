use super::*;

impl WorthQueryApplicationProjectedField {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        projection: &WorthQueryInstalledGraphProjection,
        value: AspectValue,
    ) -> Self {
        Self {
            result_path: projection.result_path_identity(),
            slot_type: projection.slot_type_identity(),
            slot_key: projection.slot_key_identity(),
            value,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn result_path(
        &self,
    ) -> &str {
        &self.result_path
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn slot_key_identity(
        &self,
    ) -> &Arc<ApplicationQueryResultSlotKey> {
        &self.slot_key
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub(super) fn retained_bytes(&self) -> usize {
        let Self {
            result_path: _result_path,
            slot_type: _slot_type,
            slot_key: _slot_key,
            value,
        } = self;
        value.owned_allocation_capacity_bytes()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn matches<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        selector: &ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> bool
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        selector.matches_slot_key(&self.slot_key)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn matches_optional<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        selector: &ApplicationQueryOptionalResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> bool
    where
        Field: OptionalApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.slot_key.as_ref() == &selector.slot_key()
    }
}
