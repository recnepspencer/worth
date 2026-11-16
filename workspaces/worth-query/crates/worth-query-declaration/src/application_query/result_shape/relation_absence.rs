use super::*;

impl<Schema, Query, Entity, Row, ShapeBinding>
    ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Row, ShapeBinding>
where
    ShapeBinding: ApplicationStructuredValueBinding<Value = Row>,
{
    /// Select every related live record lacking this exact declared field.
    /// Unavailable records or denied field reads never count as absence.
    pub fn relation_where_absent<
        Slot,
        Relation,
        DeclaredFrom,
        DeclaredTo,
        Direction,
        Child,
        Cardinality,
        NestedResult,
        NestedBinding,
        Field,
    >(
        mut self,
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            DeclaredFrom,
            DeclaredTo,
            Direction,
            Cardinality,
        >,
        nested: ApplicationQueryResultShapeBuilder<
            Schema,
            Query,
            Child,
            NestedResult,
            NestedBinding,
        >,
        field: Field,
    ) -> Self
    where
        Direction:
            ApplicationQueryResultTraversalEndpoints<Entity, Child, DeclaredFrom, DeclaredTo>,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        NestedBinding: ApplicationStructuredValueBinding<Value = NestedResult>,
        Slot: WorthQueryPortableType,
        Field: crate::application_schema::fragment::RecordSelection<Entity = Child>,
        Field::Binding: crate::application_schema::ApplicationReadableScalarValueBinding,
    {
        self.field_expectations.push(field.expected_recipe());
        let (nested_shape, nested_fields) = nested.build().into_parts();
        self.field_expectations.extend(nested_fields);
        self.relations
            .push(ApplicationQueryResultRelation::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: relation.slot_key().query_identity(),
                    slot_type: relation.slot_key().slot_identity(),
                    relation: relation.relation().to_owned(),
                    from: relation.from().to_owned(),
                    to: relation.to().to_owned(),
                    direction: relation.direction(),
                    output_name: relation.output_name().to_owned(),
                    cardinality: relation.cardinality(),
                    predicate: Some(ApplicationQueryRelationPredicate::FieldAbsent {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                        scalar_family: Field::Binding::SCALAR_FAMILY,
                    }),
                    nested_shape,
                },
            ));
        self
    }
}
