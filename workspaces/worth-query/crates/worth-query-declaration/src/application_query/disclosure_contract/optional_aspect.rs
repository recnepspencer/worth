use super::*;
impl ApplicationQueryDisclosureContract {
    #[allow(clippy::too_many_arguments)]
    pub fn disclose_optional_aspect_field_by<
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
        DisclosureBinding,
    >(
        mut self,
        selector: crate::application_query::ApplicationQueryOptionalAspectFieldRef<
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
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        DisclosureBinding: ApplicationScalarValueBinding,
        Query: crate::application_query::ApplicationQueryMarkerIdentity<Schema> + 'static,
        Slot: crate::portable_identity::WorthQueryPortableType + 'static,
    {
        let field = FieldKey::new(selector.field())
            .expect("typed application-query fields are valid Foundational keys");
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Field {
                query_type: selector.slot_key().query_identity(),
                slot_type: selector.slot_key().slot_identity(),
                entity: selector.entity().to_owned(),
                aspect: selector.aspect().to_owned(),
                field: selector.field().to_owned(),
                output_name: selector.output_name().to_owned(),
                scalar_family: selector.scalar_family(),
                value_type: Field::Binding::IDENTITY,
                presence:
                    crate::application_query::ApplicationQueryResultFieldPresence::OptionalAspect,
                projection_mask: AspectMask::new([CanonicalFieldPath::single(field.clone())]),
                diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(field)]),
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
}
