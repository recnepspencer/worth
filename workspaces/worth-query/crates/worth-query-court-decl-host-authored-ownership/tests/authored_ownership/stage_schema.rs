//! Authored stage topology stays distinct from the body's owned realization edges.
use super::*;
pub struct PreviousStage;
pub struct StageVisible;
pub struct OwnCell;
pub struct DerivedCell;
pub struct SourceLink;
impl SourceLink {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "SourceLink",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl OperationLinks<lifecycle::CreateOp> for SourceLink {}
impl DerivedCell {
    pub fn reference<S: Scenario>() -> ApplicationDerivedRelationRef<S, Self, Node, Node> {
        ApplicationDerivedRelationRef::from_schema_identifiers("DerivedCell", "Node", "Node")
    }
}
impl OperationReads<coupled::producer::Operation> for DerivedCell {}
impl OwnCell {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "OwnCell",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl PreviousStage {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "PreviousStage",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl StageVisible {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "StageVisible",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl OperationLinks<lifecycle::CreateOp> for PreviousStage {}
impl OperationLinks<lifecycle::CreateOp> for StageVisible {}
impl OperationReads<lifecycle::RetireOp> for PreviousStage {}
impl OperationUnlinks<lifecycle::RetireOp> for PreviousStage {}
impl OperationReads<coupled::producer::Operation> for Components {}
impl OperationLinks<producer::Operation> for OwnCell {}
impl OperationLinks<producer::Operation> for StageVisible {}
impl OperationLinks<coupled::producer::Operation> for StageVisible {}
impl OperationReads<coupled::producer::Operation> for OwnCell {}
impl OperationLinks<coupled::producer::Operation> for OwnCell {}
impl OperationUnlinks<coupled::producer::Operation> for OwnCell {}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if matches!(S::MODE, 42 | 64) {
        return schema
            .relation(OwnCell::reference(), Node::reference(), Node::reference())
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                OwnCell::reference(),
            )
            .operation_read_relation(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_link(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_unlink(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            );
    }
    if S::MODE == 26 {
        return schema
            .relation(OwnCell::reference(), Node::reference(), Node::reference())
            .relation(
                StageVisible::reference(),
                Node::reference(),
                Node::reference(),
            )
            .derived_relation(
                DerivedCell::reference(),
                ApplicationDerivedRelation::relation(OwnCell::reference()),
                ApplicationRelationGraphLaws::unbounded(),
            )
            .derived_components(ApplicationDerivedComponentSetDefinition::new(
                Components::reference(),
                DerivedCell::reference(),
            ))
            .operation_read_derived_relation(
                coupled::producer::Operation::reference(),
                DerivedCell::reference(),
            )
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                OwnCell::reference(),
            )
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                StageVisible::reference(),
            )
            .operation_read_relation(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_link(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_unlink(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            );
    }
    if matches!(S::MODE, 22 | 23 | 27 | 28) {
        let schema = if S::MODE == 23 {
            schema
                .derived_relation(
                    DerivedCell::reference(),
                    ApplicationDerivedRelation::relation(OwnCell::reference()).reversed(),
                    ApplicationRelationGraphLaws::unbounded(),
                )
                .operation_read_derived_relation(
                    coupled::producer::Operation::reference(),
                    DerivedCell::reference(),
                )
        } else {
            schema
        };
        let schema = if S::MODE == 28 {
            schema.operation_link(
                coupled::producer::Operation::reference(),
                StageVisible::reference(),
            )
        } else {
            schema
        };
        let schema = if S::MODE == 27 {
            schema
                .relation(
                    SourceLink::reference(),
                    Node::reference(),
                    Node::reference(),
                )
                .operation_link(lifecycle::CreateOp::reference(), SourceLink::reference())
        } else {
            schema
        };
        let path = if S::MODE == 28 {
            ApplicationQueryRootPath::from(Node::reference())
                .forward_closure(StageVisible::reference())
                .reverse_closure(OwnCell::reference())
        } else if S::MODE == 27 {
            ApplicationQueryRootPath::from(Node::reference())
                .forward(SourceLink::reference())
                .forward(StageVisible::reference())
                .where_equal(
                    Value::reference(),
                    ApplicationEncodedScalarValue::try_new(39u64).unwrap(),
                )
                .reverse(StageVisible::reference())
                .reverse(SourceLink::reference())
                .forward(StageVisible::reference())
                .reverse_closure(OwnCell::reference())
        } else {
            ApplicationQueryRootPath::from(Node::reference())
                .forward(StageVisible::reference())
                .reverse_closure(OwnCell::reference())
        };
        return schema
            .relation(OwnCell::reference(), Node::reference(), Node::reference())
            .relation(
                StageVisible::reference(),
                Node::reference(),
                Node::reference(),
            )
            .components(
                ApplicationComponentSetDefinition::new(Components::reference())
                    .coupled_by(OwnCell::reference()),
            )
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                OwnCell::reference(),
            )
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                StageVisible::reference(),
            )
            .operation_read_relation(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_link(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_unlink(
                coupled::producer::Operation::reference(),
                OwnCell::reference(),
            )
            .operation_component_read(ApplicationOperationComponentReadDefinition::new(
                coupled::producer::Operation::reference(),
                Components::reference(),
                path,
            ));
    }
    if !matches!(S::MODE, 21 | 30 | 39 | 40 | 41 | 60) {
        return schema;
    }
    let schema = if S::MODE == 40 {
        schema
            .operation_link(
                ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
                PreviousStage::reference(),
            )
            .operation_read_relation(
                coupled::producer::Operation::reference(),
                PreviousStage::reference(),
            )
            .operation_link(
                coupled::producer::Operation::reference(),
                PreviousStage::reference(),
            )
            .operation_unlink(
                coupled::producer::Operation::reference(),
                PreviousStage::reference(),
            )
    } else {
        schema
    };
    schema
        .relation(
            PreviousStage::reference(),
            Node::reference(),
            Node::reference(),
        )
        .relation(
            StageVisible::reference(),
            Node::reference(),
            Node::reference(),
        )
        .components(
            ApplicationComponentSetDefinition::new(Components::reference())
                .coupled_by(PreviousStage::reference()),
        )
        .operation_component_read(ApplicationOperationComponentReadDefinition::new(
            coupled::producer::Operation::reference(),
            Components::reference(),
            if matches!(S::MODE, 30 | 39 | 40 | 41 | 60) {
                ApplicationQueryRootPath::from(Node::reference())
                    .forward(StageVisible::reference())
                    .forward_closure(PreviousStage::reference())
            } else {
                ApplicationQueryRootPath::from(Node::reference())
                    .forward(StageVisible::reference())
                    .reverse_closure(PreviousStage::reference())
            },
        ))
}

impl OperationLinks<producer::Operation> for PreviousStage {}
impl OperationReads<coupled::producer::Operation> for PreviousStage {}
impl OperationLinks<coupled::producer::Operation> for PreviousStage {}
impl OperationUnlinks<coupled::producer::Operation> for PreviousStage {}
