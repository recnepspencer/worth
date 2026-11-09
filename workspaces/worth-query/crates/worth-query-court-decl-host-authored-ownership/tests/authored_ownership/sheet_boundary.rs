//! A single producer owns a directed sheet boundary and removes its old orbit anchor.
use super::*;
use coupled::producer::{Binding as Preserve, Operation as PreserveOp};

pub struct Successor;
impl Successor {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<
        S,
        Self,
        Occurrence<<S as Scenario>::Boundary>,
        Occurrence<<S as Scenario>::Boundary>,
    > {
        ApplicationRelationRef::from_schema_identifiers(
            "BoundaryNext",
            "Occurrence",
            "Occurrence",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(true),
                ApplicationRelationCardinality::new(
                    Some(1),
                    Some(1),
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ),
                ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
            )
            .orbit(),
        )
    }
}
macro_rules! family {
    (@entity Occurrence, $schema:ident) => { Occurrence<<$schema as Scenario>::Boundary> };
    (@entity $entity:ident, $schema:ident) => { $entity };
    ($name:ident, $entity:ident, $contract:ty, $prefix:literal, $postures:expr) => {
        pub struct $name<S>(PhantomData<fn() -> S>);
        impl<S: Scenario> WorthQueryApplicationOutputRoleFamily for $name<S> {
            type Schema = S;
            type Contract = $contract;
            type Entity = family!(@entity $entity, S);
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet = $postures;
            const MINIMUM: usize = 0;
        }
    };
}
family!(
    InitialNodes,
    Occurrence,
    producer::Outputs,
    "boundary.",
    ApplicationMutationOutputPostureSet::ALL
);
family!(
    SurvivingNodes,
    Occurrence,
    coupled::producer::Outputs,
    "boundary.",
    ApplicationMutationOutputPostureSet::ALL
);
family!(
    RetiredNodes,
    Occurrence,
    coupled::producer::Outputs,
    "retired-boundary.",
    ApplicationMutationOutputPostureSet::RETIRE
);
impl<B: Profile> OperationCreates<producer::Operation> for Occurrence<B> {}
impl<B: Profile> OperationWrites<producer::Operation> for Number<B> {}
impl OperationLinks<producer::Operation> for Successor {}
impl<B: Profile> OperationReads<PreserveOp> for Occurrence<B> {}
impl<B: Profile> OperationReads<PreserveOp> for Number<B> {}
impl OperationReads<PreserveOp> for Successor {}
impl<B: Profile> OperationCreates<PreserveOp> for Occurrence<B> {}
impl<B: Profile> OperationDeletes<PreserveOp> for Occurrence<B> {}
impl<B: Profile> OperationWrites<PreserveOp> for Number<B> {}
impl OperationLinks<PreserveOp> for Successor {}
impl OperationUnlinks<PreserveOp> for Successor {}

pub fn declare<S: Scenario>(
    s: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 69 {
        return s;
    }
    let initial =
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration();
    let preserve = PreserveOp::reference();
    let schema = s
        .operation_create(initial, Occurrence::<S::Boundary>::reference())
        .operation_write(initial, Number::<S::Boundary>::at("Number"))
        .operation_read_entity(preserve, Occurrence::<S::Boundary>::reference())
        .operation_read_field(preserve, Number::<S::Boundary>::at("Number"))
        .operation_create(preserve, Occurrence::<S::Boundary>::reference())
        .operation_delete(preserve, Occurrence::<S::Boundary>::reference())
        .operation_write(preserve, Number::<S::Boundary>::at("Number"));
    if S::Boundary::LINK_BOUNDARY {
        schema
            .operation_link(initial, Successor::reference())
            .operation_read_relation(preserve, Successor::reference())
            .operation_link(preserve, Successor::reference())
            .operation_unlink(preserve, Successor::reference())
    } else {
        schema
    }
}
pub fn initial<S: Scenario>(
    w: &mut CandidateWriter<'_, S, producer::Binding<S>>,
    source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let mut nodes = Vec::new();
    for index in 0..4u64 {
        let node = w.create_entity_in_context(
            source,
            Occurrence::<S::Boundary>::reference(),
            WorthQueryApplicationEntityKey::new(format!("boundary-{index}")).unwrap(),
        )?;
        w.initialize_field(&node, Number::<S::Boundary>::at("Number"), index)?;
        w.create_member::<InitialNodes<S>>(&index.to_string(), &node)?;
        nodes.push(node);
    }
    for index in 0..4 {
        w.link(
            Successor::reference(),
            format!("next-{index}"),
            &nodes[index],
            &nodes[(index + 1) % 4],
        )?;
    }
    Ok(())
}
pub struct Decision<S: Scenario> {
    nodes: Vec<(
        u64,
        WorthQueryInvariantMutationTarget<S, Occurrence<<S as Scenario>::Boundary>>,
    )>,
    edges: Vec<(
        WorthQueryInvariantMutationTarget<S, Occurrence<<S as Scenario>::Boundary>>,
        WorthQueryInvariantMutationTarget<S, Occurrence<<S as Scenario>::Boundary>>,
    )>,
}
pub fn read<S: Scenario>(
    r: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
) -> Result<Decision<S>, HandlerExecutionDenial> {
    let members = match r
        .prior_output_family_if_present::<Preserve<S>, SurvivingNodes<S>>()
        .map_err(HandlerExecutionDenial::new)?
    {
        Some(nodes) => nodes
            .into_iter()
            .map(|n| n.into_identity())
            .collect::<Vec<_>>(),
        None => r
            .prior_output_family::<producer::Binding<S>, InitialNodes<S>>()
            .map_err(HandlerExecutionDenial::new)?
            .into_iter()
            .map(|n| n.into_identity())
            .collect(),
    };
    let orbit = r
        .orbit(Successor::reference(), &members[0])
        .map_err(HandlerExecutionDenial::new)?;
    assert!(
        matches!(orbit.len(), 3 | 4),
        "a sheet boundary has its complete predecessor orbit"
    );
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for node in orbit.members() {
        r.entity(&node, Occurrence::<S::Boundary>::reference())?;
        let index = r
            .field(&node, Number::<S::Boundary>::at("Number"))?
            .unwrap();
        let next = r
            .related_one(Successor::reference(), &node)
            .map_err(HandlerExecutionDenial::new)?;
        edges.push((r.mutation_target(&node)?, r.mutation_target(&next)?));
        nodes.push((index, r.mutation_target(&node)?));
    }
    nodes.sort_by_key(|(index, _)| *index);
    let numbers = nodes.iter().map(|(n, _)| *n).collect::<Vec<_>>();
    assert!(numbers == [0, 1, 2, 3] || numbers == [1, 2, 4]);
    Ok(Decision { nodes, edges })
}
pub fn trim<S: Scenario>(
    w: &mut CandidateWriter<'_, S, Preserve<S>>,
    decision: Decision<S>,
    source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if decision.nodes.len() == 3 {
        for (index, target) in decision.nodes {
            let node = w.projected_entity(&target)?;
            w.preserve_member::<SurvivingNodes<S>>(&index.to_string(), &node)?;
        }
        return Ok(());
    }
    for (a, b) in decision.edges {
        let a = w.projected_entity(&a)?;
        let b = w.projected_entity(&b)?;
        w.unlink(Successor::reference(), &a, &b)?;
    }
    let mut live = Vec::new();
    for (index, target) in decision.nodes {
        let node = w.projected_entity(&target)?;
        if index == 0 || index == 3 {
            w.delete_entity(Occurrence::<S::Boundary>::reference(), &node)?;
            w.retire_member::<RetiredNodes<S>>(&index.to_string(), &node)?;
        } else {
            w.preserve_member::<SurvivingNodes<S>>(&index.to_string(), &node)?;
            live.push(node);
        }
    }
    let node = w.create_entity_in_context(
        source,
        Occurrence::<S::Boundary>::reference(),
        WorthQueryApplicationEntityKey::new("diagonal-occurrence").unwrap(),
    )?;
    w.initialize_field(&node, Number::<S::Boundary>::at("Number"), 4)?;
    w.create_member::<SurvivingNodes<S>>("4", &node)?;
    live.push(node);
    for i in 0..3 {
        w.link(
            Successor::reference(),
            format!("trimmed-next-{i}"),
            &live[i],
            &live[(i + 1) % 3],
        )?;
    }
    Ok(())
}
mod court;

pub struct Absent;
pub struct Owned;
pub trait Profile: Sized + 'static {
    type Relations: 'static;
    const UNIQUE: bool = false;
    const LINK_BOUNDARY: bool = true;
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial>;
    type Decision<S: Scenario>;
    fn initial<S: Scenario>(
        _: &mut CandidateWriter<'_, S, producer::Binding<S>>,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Ok(())
    }
    fn read<S: Scenario>(
        _: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    ) -> Result<Option<Self::Decision<S>>, HandlerExecutionDenial> {
        Ok(None)
    }
    fn trim<S: Scenario>(
        _: &mut CandidateWriter<'_, S, Preserve<S>>,
        decision: Option<Self::Decision<S>>,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        assert!(decision.is_none());
        Ok(())
    }
}
impl Profile for Absent {
    type Relations = ();
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
        if S::MODE == 43 {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, true>(
                PhantomData,
            ))?
            .build()
        } else {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, false>(
                PhantomData,
            ))?
            .build()
        }
    }
    type Decision<S: Scenario> = ();
}
impl Profile for Owned {
    type Relations = (Successor,);
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
        if S::MODE == 43 {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, true>(
                PhantomData,
            ))?
            .build()
        } else {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, false>(
                PhantomData,
            ))?
            .build()
        }
    }
    type Decision<S: Scenario> = Decision<S>;
    fn initial<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, producer::Binding<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        initial(writer, source)
    }
    fn read<S: Scenario>(
        reader: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    ) -> Result<Option<Self::Decision<S>>, HandlerExecutionDenial> {
        read(reader).map(Some)
    }
    fn trim<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, Preserve<S>>,
        decision: Option<Self::Decision<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        trim(writer, decision.expect("owned boundary decision"), source)
    }
}

pub(super) mod split_walk;

pub use super::native_boundary::{BoundaryOccurrence as Occurrence, Number};
