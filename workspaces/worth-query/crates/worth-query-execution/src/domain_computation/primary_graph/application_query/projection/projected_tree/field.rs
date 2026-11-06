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
            content: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn source(
        projection: &WorthQueryInstalledGraphProjection,
        value: AspectValue,
        content: Option<worth_relational::facade::content::RelationalContentFrame>,
    ) -> Result<Self, worth_relational::facade::content::RelationalContentDenial> {
        use worth_relational::facade::content::RelationalContentDenial as Denial;
        match (&value, &content) {
            (AspectValue::ContentRef(identity), Some(frame)) if *identity == frame.identity() => {}
            (AspectValue::ContentRef(_), Some(_)) => return Err(Denial::EncodingInconsistency),
            (AspectValue::ContentRef(_), None) => return Err(Denial::MissingContent),
            (_, Some(_)) => return Err(Denial::EncodingInconsistency),
            (_, None) => {}
        }
        let mut field = Self::new(projection, value);
        field.content = content;
        Ok(field)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn content(
        &self,
    ) -> Option<&worth_relational::facade::content::RelationalContentFrame> {
        self.content.as_ref()
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
            content: _,
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
    >(
        &self,
        selector: &ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>,
    ) -> bool
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,

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
    >(
        &self,
        selector: &ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>,
    ) -> bool
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,

        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.slot_key.as_ref() == &selector.slot_key()
    }
}
