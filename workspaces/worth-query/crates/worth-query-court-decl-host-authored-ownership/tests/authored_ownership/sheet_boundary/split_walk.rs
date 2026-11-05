//! Retiring an owned fork root preserves observation of both surviving regions.
use super::*;
use currentness_guards::RETIRE_FORK_ROOT;
mod command;
mod court;
pub use command::{Extend, ExtendBinding, Handler};
// The orbit anchor's `ForeignNext` cardinality binds every `Occurrence`, so the
// fork lives in its own entity kind.
worth_query_entity!(pub Fork for S: Scenario);
worth_query_aspect!(pub ForkPosition for S: Scenario, Fork; identity = AspectIdentity(0x919B3971), revision = AspectContractRevision(1));
worth_query_field!(pub ForkNumber for S: Scenario, Fork, ForkPosition: u64 => U64ApplicationValueBinding, read_only, equality);
family!(
    InitialFork,
    Fork,
    producer::Outputs,
    "fork.",
    ApplicationMutationOutputPostureSet::ALL
);
family!(
    SurvivingFork,
    Fork,
    coupled::producer::Outputs,
    "fork.",
    ApplicationMutationOutputPostureSet::ALL
);
family!(
    RetiredFork,
    Fork,
    coupled::producer::Outputs,
    "retired-fork.",
    ApplicationMutationOutputPostureSet::RETIRE
);
pub struct Branch;
impl Branch {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Fork, Fork> {
        ApplicationRelationRef::from_schema_identifiers(
            "Branch",
            "Fork",
            "Fork",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl OperationCreates<producer::Operation> for Fork {}
impl OperationWrites<producer::Operation> for ForkNumber {}
impl OperationLinks<producer::Operation> for Branch {}
impl OperationReads<PreserveOp> for Fork {}
impl OperationReads<PreserveOp> for ForkNumber {}
impl OperationDeletes<PreserveOp> for Fork {}
impl OperationReads<PreserveOp> for Branch {}
impl OperationUnlinks<PreserveOp> for Branch {}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let initial =
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration();
    let preserve = PreserveOp::reference();
    command::declare(
        schema
            .entity(Fork::reference())
            .aspect(Fork::reference(), ForkPosition::reference())
            .unique_field(Fork::reference(), ForkNumber::reference())
            .relation(Branch::reference(), Fork::reference(), Fork::reference())
            .operation_create(initial, Fork::reference())
            .operation_write(initial, ForkNumber::reference())
            .operation_link(initial, Branch::reference())
            .operation_read_entity(preserve, Fork::reference())
            .operation_read_field(preserve, ForkNumber::reference())
            .operation_read_relation(preserve, Branch::reference())
            .operation_delete(preserve, Fork::reference())
            .operation_unlink(preserve, Branch::reference()),
    )
}
pub fn initial<S: Scenario>(
    writer: &mut CandidateWriter<'_, S, producer::Binding<S>>,
    source: &WorthQueryApplicationEffectEntity<S, Node>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let mut nodes = Vec::new();
    for index in 0..3u64 {
        let node = writer.create_entity_in_context(
            source,
            Fork::reference(),
            WorthQueryApplicationEntityKey::new(format!("fork-{index}")).unwrap(),
        )?;
        writer.initialize_field(&node, ForkNumber::reference(), index)?;
        writer.create_member::<InitialFork<S>>(&index.to_string(), &node)?;
        nodes.push(node);
    }
    for index in 1..3 {
        writer.link(
            Branch::reference(),
            format!("branch-{index}"),
            &nodes[0],
            &nodes[index],
        )?;
    }
    Ok(())
}
type Target<S> = WorthQueryInvariantMutationTarget<S, Fork>;
pub struct Decision<S> {
    nodes: Vec<(u64, Target<S>)>,
    retired_root_edges: Vec<(Target<S>, Target<S>)>,
}
/// Observes the fork's walk from its root, or from each surviving leaf once
/// the root has retired; the root's edges are read only when this edit
/// retires it.
pub fn read<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    value: Option<u64>,
) -> Result<Decision<S>, HandlerExecutionDenial> {
    let members = match reader
        .prior_output_family_if_present::<Preserve<S>, SurvivingFork<S>>()
        .map_err(HandlerExecutionDenial::new)?
    {
        Some(members) => members
            .into_iter()
            .map(|m| m.into_identity())
            .collect::<Vec<_>>(),
        None => reader
            .prior_output_family::<producer::Binding<S>, InitialFork<S>>()
            .map_err(HandlerExecutionDenial::new)?
            .into_iter()
            .map(|m| m.into_identity())
            .collect(),
    };
    let mut numbered = Vec::new();
    for member in members {
        numbered.push((
            reader.field(&member, ForkNumber::reference())?.unwrap(),
            member,
        ));
    }
    numbered.sort_by_key(|(number, _)| *number);
    let numbers = numbered.iter().map(|(n, _)| *n).collect::<Vec<_>>();
    let mut retired_root_edges = Vec::new();
    if numbers == [0, 1, 2] {
        let root = &numbered[0].1;
        let walk = reader
            .walk(Branch::reference(), RelationDirection::Forward, root)
            .map_err(HandlerExecutionDenial::new)?;
        assert_eq!(walk.len(), 3);
        if value == Some(RETIRE_FORK_ROOT) {
            for edge in reader
                .relations_from(Branch::reference(), root)
                .map_err(HandlerExecutionDenial::new)?
            {
                retired_root_edges.push((
                    reader.mutation_target(edge.from())?,
                    reader.mutation_target(edge.to())?,
                ));
            }
        }
    } else {
        assert_eq!(numbers, [1, 2], "a retired root leaves both leaves");
        for (_, root) in &numbered {
            reader
                .walk(Branch::reference(), RelationDirection::Forward, root)
                .map_err(HandlerExecutionDenial::new)?;
        }
    }
    let mut nodes = Vec::new();
    for (number, member) in numbered {
        nodes.push((number, reader.mutation_target(&member)?));
    }
    Ok(Decision {
        nodes,
        retired_root_edges,
    })
}
pub fn shrink<S: Scenario>(
    writer: &mut CandidateWriter<'_, S, Preserve<S>>,
    decision: Decision<S>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let retire_root = !decision.retired_root_edges.is_empty();
    for (from, to) in decision.retired_root_edges {
        let from = writer.projected_entity(&from)?;
        let to = writer.projected_entity(&to)?;
        writer.unlink(Branch::reference(), &from, &to)?;
    }
    for (number, target) in decision.nodes {
        let node = writer.projected_entity(&target)?;
        if retire_root && number == 0 {
            writer.delete_entity(Fork::reference(), &node)?;
            writer.retire_member::<RetiredFork<S>>("0", &node)?;
        } else {
            writer.preserve_member::<SurvivingFork<S>>(&number.to_string(), &node)?;
        }
    }
    Ok(())
}
