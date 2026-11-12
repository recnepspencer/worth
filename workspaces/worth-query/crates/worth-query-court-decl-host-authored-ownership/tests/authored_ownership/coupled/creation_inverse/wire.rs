//! A generic wire roster: four vertices, three edges, six definitions, no face records.
use super::super::super::*;
use coupled::producer::{Binding as Preserve, Operation as PreserveOp};
use lifecycle::{RetireBinding, RetireOp, RetiredOutputs};
use sheet_boundary::{InitialNodes, Number, Occurrence, Profile, RetiredNodes};
use worth_query_decl::facade::application_schema::fragment;

pub struct Wire;
pub struct Input;
impl fragment::Relation for Input {
    const NAME: &'static str = "WireInput";
    type To = NativeNode<true>;
}
impl Input {
    fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Occurrence<S::Boundary>, Node<S>>
    {
        ApplicationRelationRef::from_schema_identifiers(
            "WireInput",
            "Occurrence",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl OperationLinks<producer::Operation> for Input {}
impl OperationReads<PreserveOp> for Input {}
impl OperationUnlinks<PreserveOp> for Input {}
impl OperationReads<RetireOp> for Input {}
impl OperationUnlinks<RetireOp> for Input {}
impl<B: Profile> OperationReads<RetireOp> for Occurrence<B> {}
impl<B: Profile> OperationReads<RetireOp> for Number<B> {}
impl<B: Profile> OperationDeletes<RetireOp> for Occurrence<B> {}
pub struct RetiredMembers<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRoleFamily for RetiredMembers<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Occurrence<S::Boundary>;
    const PREFIX: &'static str = "retired-wire.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::RETIRE;
    const MINIMUM: usize = 0;
}
pub struct Coupling;
impl Coupling {
    fn reference<S: Scenario>(
    ) -> ApplicationDerivedRelationRef<S, Self, Occurrence<S::Boundary>, Occurrence<S::Boundary>>
    {
        ApplicationDerivedRelationRef::from_schema_identifiers(
            "WireCoupling",
            "Occurrence",
            "Occurrence",
        )
    }
}
pub struct Components;
impl Components {
    fn reference<S: Scenario>() -> ApplicationComponentSetRef<S, Self, Occurrence<S::Boundary>> {
        ApplicationComponentSetRef::new("WireComponents", Occurrence::<S::Boundary>::reference())
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 72 {
        return schema;
    }
    let initial =
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration();
    // Each occurrence follows its retained source into its publication, then
    // through the authored input. The body's Peer edge is deferred on undo.
    let input = || {
        ApplicationDerivedRelation::relation(Input::reference())
            .then(ApplicationDerivedRelation::relation(
                StageVisible::reference(),
            ))
            .then(ApplicationDerivedRelation::relation(OwnCell::reference()).reversed())
            .then(ApplicationDerivedRelation::relation(Peer::reference()))
    };
    schema
        .operation_read_entity(
            RetireOp::reference(),
            Occurrence::<S::Boundary>::reference(),
        )
        .operation_read_field(RetireOp::reference(), Number::<S::Boundary>::at("Number"))
        .operation_read_relation(RetireOp::reference(), Input::reference())
        .operation_unlink(RetireOp::reference(), Input::reference())
        .operation_delete(
            RetireOp::reference(),
            Occurrence::<S::Boundary>::reference(),
        )
        .derived_relation(
            Coupling::reference(),
            input().then(input().reversed()),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .derived_components(ApplicationDerivedComponentSetDefinition::new(
            Components::reference(),
            Coupling::reference(),
        ))
        .operation_create(initial, Occurrence::<S::Boundary>::reference())
        .operation_write(initial, Number::<S::Boundary>::at("Number"))
        .operation_link(initial, Input::reference())
        .operation_link(initial, StageVisible::reference())
        .operation_link(initial, OwnCell::reference())
        .operation_read_entity(
            PreserveOp::reference(),
            Occurrence::<S::Boundary>::reference(),
        )
        .operation_read_field(PreserveOp::reference(), Number::<S::Boundary>::at("Number"))
        .operation_read_relation(PreserveOp::reference(), Input::reference())
        .operation_unlink(PreserveOp::reference(), Input::reference())
        .operation_delete(
            PreserveOp::reference(),
            Occurrence::<S::Boundary>::reference(),
        )
        .operation_read_relation(PreserveOp::reference(), OwnCell::reference())
        .operation_unlink(PreserveOp::reference(), OwnCell::reference())
        .operation_link(PreserveOp::reference(), OwnCell::reference())
}
pub struct Member<S: Scenario> {
    occurrence: WorthQueryInvariantMutationTarget<S, Occurrence<S::Boundary>>,
    owner: WorthQueryInvariantMutationTarget<S, Node<S>>,
    suffix: String,
}
impl Profile for Wire {
    type Relations = (Input,);
    type Decision<S: Scenario> = Vec<Member<S>>;
    fn read_retirement<S: Scenario>(
        reader: &mut DecisionReader<'_, '_, '_, S, RetireBinding<S>>,
        id: u64,
    ) -> Result<Vec<Member<S>>, HandlerExecutionDenial> {
        read_retirement(reader, id)
    }
    fn retire_created<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, RetireBinding<S>>,
        members: Vec<Member<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        retire_created(writer, members)
    }
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, false>(
            PhantomData,
        ))?
        .build()
    }
    fn initial<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, producer::Binding<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        for (index, suffix) in (0..4)
            .map(|i| format!("vertex-{i}"))
            .chain((0..3).map(|i| format!("edge-{i}")))
            .chain((0..6).map(|i| format!("definition-{i}")))
            .enumerate()
        {
            let member = writer.create_entity_in_context(
                source,
                Occurrence::<S::Boundary>::reference(),
                WorthQueryApplicationEntityKey::new(&suffix).unwrap(),
            )?;
            writer.initialize_field(&member, Number::<S::Boundary>::at("Number"), index as u64)?;
            writer.link(
                Input::reference(),
                format!("input-{suffix}"),
                &member,
                source,
            )?;
            writer.create_member::<InitialNodes<S>>(&suffix, &member)?;
        }
        Ok(())
    }
    fn read<S: Scenario>(
        reader: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    ) -> Result<Option<Self::Decision<S>>, HandlerExecutionDenial> {
        let members = match reader
            .prior_output_family_if_present::<Preserve<S>, sheet_boundary::SurvivingNodes<S>>()
            .map_err(HandlerExecutionDenial::new)?
        {
            Some(members) => members
                .into_iter()
                .map(|m| (m.role().to_owned(), m.into_identity()))
                .collect::<Vec<_>>(),
            None => reader
                .prior_output_family::<producer::Binding<S>, InitialNodes<S>>()
                .map_err(HandlerExecutionDenial::new)?
                .into_iter()
                .map(|m| (m.role().to_owned(), m.into_identity()))
                .collect(),
        };
        let mut targets = Vec::new();
        for (role, member) in members {
            let suffix = role
                .strip_prefix("boundary.")
                .expect("declared family prefix")
                .to_owned();
            reader.entity(&member, Occurrence::<S::Boundary>::reference())?;
            reader.field(&member, Number::<S::Boundary>::at("Number"))?;
            let edges = reader
                .relations_from(Input::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(edges.len(), 1, "each wire member has one actual input");
            targets.push(Member {
                occurrence: reader.mutation_target(&member)?,
                owner: reader.mutation_target(edges[0].to())?,
                suffix,
            });
        }
        assert_eq!(targets.len(), 13, "the actual wire roster is complete");
        Ok(Some(targets))
    }
    fn trim<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, Preserve<S>>,
        decision: Option<Self::Decision<S>>,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        for member in decision.expect("observed roster") {
            let occurrence = writer.projected_entity(&member.occurrence)?;
            let owner = writer.projected_entity(&member.owner)?;
            writer.unlink(Input::reference(), &occurrence, &owner)?;
            writer.delete_entity(Occurrence::<S::Boundary>::reference(), &occurrence)?;
            writer.retire_member::<RetiredNodes<S>>(&member.suffix, &occurrence)?;
        }
        Ok(())
    }
}

/// Observe the producer's actual generated roster before the authored inverse.
pub fn read_retirement<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, RetireBinding<S>>,
    id: u64,
) -> Result<Vec<Member<S>>, HandlerExecutionDenial> {
    let source = reader.resolve_entity(schema::id::<S>(), id)?;
    let mut members = Vec::new();
    for edge in reader
        .relations_to(Input::reference(), &source)
        .map_err(HandlerExecutionDenial::new)?
    {
        reader.entity(edge.from(), Occurrence::<S::Boundary>::reference())?;
        let number = reader
            .field(edge.from(), Number::<S::Boundary>::at("Number"))?
            .expect("a wire member has its ordinal");
        let suffix = match number {
            0..=3 => format!("vertex-{number}"),
            4..=6 => format!("edge-{}", number - 4),
            7..=12 => format!("definition-{}", number - 7),
            _ => panic!("the fixture has exactly thirteen wire members"),
        };
        members.push(Member {
            occurrence: reader.mutation_target(edge.from())?,
            owner: reader.mutation_target(&source)?,
            suffix,
        });
    }
    assert_eq!(members.len(), 13, "retire the exact live owned roster");
    Ok(members)
}
pub fn retire_created<S: Scenario>(
    writer: &mut CandidateWriter<'_, S, RetireBinding<S>>,
    members: Vec<Member<S>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for member in members {
        let occurrence = writer.projected_entity(&member.occurrence)?;
        let owner = writer.projected_entity(&member.owner)?;
        writer.unlink(Input::reference(), &occurrence, &owner)?;
        writer.delete_entity(Occurrence::<S::Boundary>::reference(), &occurrence)?;
        writer.retire_member::<RetiredMembers<S>>(&member.suffix, &occurrence)?;
    }
    Ok(())
}
