//! Component fixtures share installed bindings while varying only the disputed contract.
use super::*;
fn path<S: OrbitModel>() -> ApplicationQueryRootPath<S, Node, Node> {
    ApplicationQueryRootPath::from(Node::reference()).forward_closure(Visible::reference())
}
fn builder<S, Q>(
    query: ApplicationQueryReference<S, Q, Parameters, NodeRow, Node>,
    disclosure: ApplicationQueryDisclosureContract,
) -> ApplicationQueryDefinitionBuilder<S, Q, Parameters, NodeRow, Node>
where
    S: OrbitModel,
    Q: ApplicationQueryMarkerIdentity<
            S,
            Scope = Node,
            ResultBinding = RowsBinding,
            ParameterBinding = ParametersBinding,
        > + 'static,
{
    ApplicationQueryDefinitionBuilder::declare(query)
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<S, Q, Node, NodeRow, RowsBinding>::new(
                Node::reference(),
            )
            .field(key::<S, Q>())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
fn disclosure<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S, Scope = Node> + 'static>(
    value: Option<u64>,
) -> ApplicationQueryDisclosureContract {
    disclosure_with_influence::<S, Q>(value, ApplicationQueryInfluenceContract::permit_all())
}
fn disclosure_with_influence<
    S: OrbitModel,
    Q: ApplicationQueryMarkerIdentity<S, Scope = Node> + 'static,
>(
    value: Option<u64>,
    influence: ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let contract = ApplicationQueryDisclosureContract::governed_by(
        "component-query",
        super::super::capability::declarations::ReadOrbit::<S>::reference(),
    )
    .disclose_field_by(
        key::<S, Q>(),
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap(),
        ApplicationQueryInfluenceContract::forbid_all(),
    );
    match value {
        None => contract,
        Some(value) => contract.components_by(
            selection::<S, Q>(),
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(value).unwrap(),
            influence,
        ),
    }
}
pub(super) fn declare<const O: bool>(
    schema: ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
) -> ApplicationSchemaDeclarationBuilder<OrbitSchema<O>> {
    let public = ApplicationQueryDisclosureContract::public;
    let metadata = ApplicationQueryDefinitionBuilder::declare(Components::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                OrbitSchema<O>,
                Components,
                Node,
                metadata_only::Member,
                metadata_only::MemberBinding,
            >::new(Node::reference())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(path())
        .component_selection(selection::<OrbitSchema<O>, Components>())
        .build()
        .unwrap();
    schema
        .application_query(metadata)
        .application_query(
            builder(Census::reference(), public())
                .root_path(path())
                .component_selection(selection::<OrbitSchema<O>, Census>())
                .build()
                .unwrap(),
        )
        .application_query(
            builder(Of::reference(), public())
                .root_path(path())
                .component_selection(of::<OrbitSchema<O>>())
                .build()
                .unwrap(),
        )
        .application_query(
            builder(Partial::reference(), public())
                .root_path(ApplicationQueryRootPath::from(Node::reference()))
                .component_selection(selection::<OrbitSchema<O>, Partial>())
                .build()
                .unwrap(),
        )
        .application_query(
            builder(Empty::reference(), public())
                .root_path(
                    path().where_equal(
                        Marked::reference(),
                        ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                            .unwrap(),
                    ),
                )
                .component_selection(selection::<OrbitSchema<O>, Empty>())
                .build()
                .unwrap(),
        )
        .application_query(
            builder(
                Allowed::reference(),
                disclosure::<OrbitSchema<O>, Allowed>(Some(0)),
            )
            .root_path(path())
            .component_selection(selection::<OrbitSchema<O>, Allowed>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                Omitted::reference(),
                disclosure::<OrbitSchema<O>, Omitted>(Some(1)),
            )
            .root_path(path())
            .component_selection(selection::<OrbitSchema<O>, Omitted>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                Missing::reference(),
                disclosure::<OrbitSchema<O>, Missing>(None),
            )
            .root_path(path())
            .component_selection(selection::<OrbitSchema<O>, Missing>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                GuardDenied::reference(),
                disclosure::<OrbitSchema<O>, GuardDenied>(Some(0)),
            )
            .root_path(
                path().where_equal(
                    Marked::reference(),
                    ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                        .unwrap(),
                ),
            )
            .component_selection(selection::<OrbitSchema<O>, GuardDenied>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                GuardAllowed::reference(),
                disclosure::<OrbitSchema<O>, GuardAllowed>(Some(0)).use_field_by(
                    Marked::reference::<OrbitSchema<O>>(),
                    ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0)
                        .unwrap(),
                    ApplicationQueryInfluenceContract::permit_all(),
                ),
            )
            .root_path(
                path().where_equal(
                    Marked::reference(),
                    ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                        .unwrap(),
                ),
            )
            .component_selection(selection::<OrbitSchema<O>, GuardAllowed>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                AggregateDenied::reference(),
                disclosure_with_influence::<OrbitSchema<O>, AggregateDenied>(
                    Some(0),
                    ApplicationQueryInfluenceContract::permit([
                        ApplicationQueryObservableInfluence::RowPresence,
                        ApplicationQueryObservableInfluence::Count,
                        ApplicationQueryObservableInfluence::Ordering,
                    ]),
                ),
            )
            .root_path(path())
            .component_selection(selection::<OrbitSchema<O>, AggregateDenied>())
            .build()
            .unwrap(),
        )
        .application_query(
            builder(
                OrderingDenied::reference(),
                disclosure_with_influence::<OrbitSchema<O>, OrderingDenied>(
                    Some(0),
                    ApplicationQueryInfluenceContract::permit([
                        ApplicationQueryObservableInfluence::RowPresence,
                        ApplicationQueryObservableInfluence::Count,
                        ApplicationQueryObservableInfluence::Aggregate,
                    ]),
                ),
            )
            .root_path(path())
            .component_selection(selection::<OrbitSchema<O>, OrderingDenied>())
            .build()
            .unwrap(),
        )
}
