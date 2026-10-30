//! One explicit parent-optional result slot; the schema child stays Required.
use super::*;
impl<Schema, Query, Entity, Row, Binding>
    ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Row, Binding>
where
    Binding: ApplicationStructuredValueBinding<Value = Row>,
{
    pub fn optional_aspect_field<Slot, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: crate::application_query::ApplicationQueryOptionalAspectFieldRef<
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
    ) -> Self
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.fields
            .push(ApplicationQueryResultField::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultFieldParts {
                    query_type: field.slot_key().query_identity(),
                    slot_type: field.slot_key().slot_identity(),
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                    output_name: field.output_name().to_owned(),
                    scalar_family: Field::Binding::SCALAR_FAMILY,
                    value_type: Field::Binding::IDENTITY,
                    presence: ApplicationQueryResultFieldPresence::OptionalAspect,
                },
            ));
        self
    }
}
