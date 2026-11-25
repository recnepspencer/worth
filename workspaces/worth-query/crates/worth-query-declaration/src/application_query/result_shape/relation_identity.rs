use super::*;
use crate::application_schema::EntityRefApplicationValueBinding;
use crate::archive_decode_authority::ARCHIVE_DECODE;

impl<Schema, Query, Entity, Row, ShapeBinding>
    ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Row, ShapeBinding>
where
    ShapeBinding: ApplicationStructuredValueBinding<Value = Row>,
{
    /// Select the exact full-generation endpoint inside this admitted relation.
    /// A coordinate cannot widen the root or relation's read authority.
    pub fn relation_where_entity<
        Slot,
        Relation,
        From,
        To,
        Direction,
        Child,
        Cardinality,
        NestedResult,
        NestedBinding,
        Parameter,
    >(
        mut self,
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
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
        parameter: ApplicationQueryParameterRef<Query, Parameter, EntityRefApplicationValueBinding>,
    ) -> Self
    where
        Direction: ApplicationQueryResultTraversalEndpoints<Entity, Child, From, To>,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        NestedBinding: ApplicationStructuredValueBinding<Value = NestedResult>,
        Slot: WorthQueryPortableType,
    {
        let (nested_shape, nested_fields) = nested.build().into_parts();
        self.field_expectations.extend(nested_fields);
        let predicate = ApplicationQueryRelationPredicate::EntityEqualParameter {
            entity: nested_shape.root_entity().to_owned(),
            parameter: parameter.name().to_owned(),
        };
        self.relations
            .push(ApplicationQueryResultRelation::from_untrusted_parts(
                ARCHIVE_DECODE,
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: relation.slot_key().query_identity(),
                    slot_type: relation.slot_key().slot_identity(),
                    relation: relation.relation().to_owned(),
                    from: relation.from().to_owned(),
                    to: relation.to().to_owned(),
                    direction: relation.direction(),
                    output_name: relation.output_name().to_owned(),
                    cardinality: relation.cardinality(),
                    predicate: Some(predicate),
                    nested_shape,
                },
            ));
        self
    }
}
