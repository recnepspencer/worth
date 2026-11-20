//! Query definitions for the self-referential managed-index Source occurrence schema.
use super::*;

type ShapeOf<Q, E> = ApplicationQueryResultShapeBuilder<SelfRefSchema, Q, E, RowValue, RowBinding>;

pub(super) fn zero_source_query(
) -> ApplicationQueryDefinitionBuilder<SelfRefSchema, ZeroSourceQuery, Params, RowValue, Scope> {
    let shape = ShapeOf::<ZeroSourceQuery, ZeroNode>::new(entity(SelfRefSchema, ZeroNode))
        .relation(
            ApplicationQueryResultRelationRef::<_, ZeroSelfSlot, _, _, _, _, _, _>::forward_many(
                "self_links",
                relation(SelfRefSchema, ZeroNode, ZeroSelf),
            ),
            ShapeOf::<ZeroSourceQuery, ZeroNode>::new(entity(SelfRefSchema, ZeroNode)),
        )
        .relation(
            ApplicationQueryResultRelationRef::<_, ZeroTagSlot, _, _, _, _, _, _>::forward_many(
                "tags",
                relation(SelfRefSchema, ZeroNode, ZeroTag),
            ),
            ShapeOf::<ZeroSourceQuery, Tag>::new(entity(SelfRefSchema, Tag)),
        )
        .build();
    ApplicationQueryDefinitionBuilder::declare(ZeroSourceQuery::reference())
        .root(entity(SelfRefSchema, ZeroNode))
        .scope(entity(SelfRefSchema, Scope))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(entity(SelfRefSchema, Scope)).forward(relation(
                SelfRefSchema,
                Scope,
                ContainsZero,
            )),
        )
}

pub(super) fn fielded_source_query(
) -> ApplicationQueryDefinitionBuilder<SelfRefSchema, FieldedSourceQuery, Params, RowValue, Scope> {
    let shape = ShapeOf::<FieldedSourceQuery, FieldedNode>::new(entity(SelfRefSchema, FieldedNode))
        .field(
            ApplicationQueryResultFieldRef::<_, RootLabelSlot, _, _>::new(
                "label",
                LabelField::at("Label"),
            ),
        )
        .relation(
            ApplicationQueryResultRelationRef::<_, FieldedSelfSlot, _, _, _, _, _, _>::forward_many(
                "self_links",
                relation(SelfRefSchema, FieldedNode, FieldedSelf),
            ),
            ShapeOf::<FieldedSourceQuery, FieldedNode>::new(entity(SelfRefSchema, FieldedNode))
                .field(
                    ApplicationQueryResultFieldRef::<_, ChildLabelSlot, _, _>::new(
                        "label",
                        LabelField::at("Label"),
                    ),
                ),
        )
        .relation(
            ApplicationQueryResultRelationRef::<_, FieldedTagSlot, _, _, _, _, _, _>::forward_many(
                "tags",
                relation(SelfRefSchema, FieldedNode, FieldedTag),
            ),
            ShapeOf::<FieldedSourceQuery, Tag>::new(entity(SelfRefSchema, Tag)),
        )
        .build();
    ApplicationQueryDefinitionBuilder::declare(FieldedSourceQuery::reference())
        .root(entity(SelfRefSchema, FieldedNode))
        .scope(entity(SelfRefSchema, Scope))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(entity(SelfRefSchema, Scope)).forward(relation(
                SelfRefSchema,
                Scope,
                ContainsFielded,
            )),
        )
}
