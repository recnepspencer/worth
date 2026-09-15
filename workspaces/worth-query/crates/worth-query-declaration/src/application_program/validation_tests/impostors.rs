use super::*;

struct ImpostorSource;
struct ImpostorOutput;
struct ImpostorSourceToTarget;
feature!(ImpostorSource, "source");
impl ApplicationOutputPort<Schema, ImpostorSource> for ImpostorOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "output";
}
impl ApplicationConnectionIdentity for ImpostorSourceToTarget {
    const IDENTITY: &'static str = "impostor-source-to-target";
}
impl ApplicationOccurrenceConnectionBinding<Schema, ImpostorSource, Target>
    for ImpostorSourceToTarget
{
}
type ImpostorFeatureConnection = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<
        Schema,
        ImpostorSource,
        ImpostorOutput,
        Target,
        TargetInput,
        ImpostorSourceToTarget,
    >,
>;
program!(
    SameNameForeignFeature,
    (SourceNode, TargetNode),
    (ImpostorFeatureConnection,),
    (InventoryNode,)
);

struct ImpostorTarget;
struct ImpostorTargetInput;
struct ImpostorTargetConnection;
feature!(ImpostorTarget, "target");
impl ApplicationInputPort<Schema, ImpostorTarget> for ImpostorTargetInput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for ImpostorTargetConnection {
    const IDENTITY: &'static str = "source-to-impostor-target";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Source, ImpostorTarget>
    for ImpostorTargetConnection
{
}
type ImpostorTargetConnectionNode = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<
        Schema,
        Source,
        SourceOutput,
        ImpostorTarget,
        ImpostorTargetInput,
        ImpostorTargetConnection,
    >,
>;
program!(
    SameNameForeignTarget,
    (SourceNode, TargetNode),
    (ImpostorTargetConnectionNode,),
    (InventoryNode,)
);

struct ImpostorSourceOutput;
struct ImpostorPortToTarget;
impl ApplicationOutputPort<Schema, Source> for ImpostorSourceOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "output";
}
impl ApplicationConnectionIdentity for ImpostorPortToTarget {
    const IDENTITY: &'static str = "impostor-port-to-target";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Source, Target> for ImpostorPortToTarget {}
type ImpostorPortConnection = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<
        Schema,
        Source,
        ImpostorSourceOutput,
        Target,
        TargetInput,
        ImpostorPortToTarget,
    >,
>;
program!(
    SameNameForeignPort,
    (SourceNode, TargetNode),
    (ImpostorPortConnection,),
    (InventoryNode,)
);

type DuplicateOutputInventoryNode = ApplicationProgramInventory<Inventory, (Output, Output)>;
program!(
    DuplicateInventoryOutput,
    (SourceNode, TargetNode),
    (Connection,),
    (DuplicateOutputInventoryNode,)
);

struct ImpostorTargetOutput;
impl ApplicationOutputPort<Schema, Target> for ImpostorTargetOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "result";
}
type ForeignTypeOutput = ApplicationProgramOutput<Target, ImpostorTargetOutput>;
type ForeignTypeInventoryNode = ApplicationProgramInventory<Inventory, (ForeignTypeOutput,)>;
program!(
    ForeignInventoryOutputType,
    (SourceNode, TargetNode),
    (Connection,),
    (ForeignTypeInventoryNode,)
);

struct UnknownTargetOutput;
impl ApplicationOutputPort<Schema, Target> for UnknownTargetOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "unknown";
}
type UnknownOutput = ApplicationProgramOutput<Target, UnknownTargetOutput>;
type UnknownInventoryNode = ApplicationProgramInventory<Inventory, (UnknownOutput,)>;
program!(
    UnknownInventoryOutputProgram,
    (SourceNode, TargetNode),
    (Connection,),
    (UnknownInventoryNode,)
);

pub(super) fn same_name_foreign_types_do_not_enter_the_graph() {
    assert_eq!(
        denial::<SameNameForeignFeature>().kind(),
        ApplicationProgramValidationDenialKind::ForeignFeatureType
    );
    assert_eq!(
        denial::<SameNameForeignPort>().kind(),
        ApplicationProgramValidationDenialKind::ForeignPortType
    );
    let target = denial::<SameNameForeignTarget>();
    assert_eq!(
        target.kind(),
        ApplicationProgramValidationDenialKind::ForeignFeatureType
    );
    assert_eq!(target.subject(), "source-to-impostor-target");
}

pub(super) fn duplicate_inventory_output_is_denied() {
    let denial = denial::<DuplicateInventoryOutput>();
    assert_eq!(
        denial.kind(),
        ApplicationProgramValidationDenialKind::DuplicateInventoryOutput
    );
    assert_eq!(denial.subject(), "complete:target.result");
}

pub(super) fn foreign_and_unknown_inventory_outputs_are_denied() {
    for denial in [
        denial::<ForeignInventoryOutputType>(),
        denial::<UnknownInventoryOutputProgram>(),
    ] {
        assert_eq!(
            denial.kind(),
            ApplicationProgramValidationDenialKind::UnknownInventoryOutput
        );
        assert!(denial.subject().starts_with("complete:target."));
    }
}
