//! A foreign selected orbit member survives the reader's partial owned shrink.
use super::*;
use coupled::producer::{Binding as Preserve, Operation as PreserveOp};
use currentness_guards::{JOIN_ORBIT, RETIRE_ANCHOR};
use sheet_boundary::{InitialNodes, Number, Occurrence, RetiredNodes, SurvivingNodes};
pub(super) mod command;
mod court;
pub use command::{Birth, BirthBinding};
pub struct Next;
impl Next {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<
        S,
        Self,
        Occurrence<<S as Scenario>::Boundary>,
        Occurrence<<S as Scenario>::Boundary>,
    > {
        ApplicationRelationRef::from_schema_identifiers(
            "ForeignNext",
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
impl OperationLinks<producer::Operation> for Next {}
impl OperationReads<PreserveOp> for Next {}
impl OperationLinks<PreserveOp> for Next {}
impl OperationUnlinks<PreserveOp> for Next {}
/// Declares the orbit relation and the foreign birth command; the shared
/// occurrence schema belongs to `currentness_guards`.
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let initial =
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration();
    let preserve = PreserveOp::reference();
    command::declare(
        schema
            .operation_link(initial, Next::reference())
            .operation_read_relation(preserve, Next::reference())
            .operation_link(preserve, Next::reference())
            .operation_unlink(preserve, Next::reference()),
    )
}
type Target<S> = WorthQueryInvariantMutationTarget<S, Occurrence<<S as Scenario>::Boundary>>;
/// `Keep` carries the anchor unchanged through edits addressed to the fork.
pub enum Decision<S: Scenario> {
    Keep(Option<Target<S>>),
    Edit {
        own: Option<Target<S>>,
        foreign: Target<S>,
        edges: Vec<(Target<S>, Target<S>)>,
        join: bool,
    },
}
pub fn initial<S: Scenario>(
    writer: &mut CandidateWriter<'_, S, producer::Binding<S>>,
    source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let own = writer.create_entity_in_context(
        source,
        Occurrence::<S::Boundary>::reference(),
        WorthQueryApplicationEntityKey::new("own-orbit-member").unwrap(),
    )?;
    writer.initialize_field(&own, Number::<S::Boundary>::at("Number"), 0)?;
    writer.link(Next::reference(), "own-self-loop", &own, &own)?;
    writer.create_member::<InitialNodes<S>>("0", &own)
}
pub fn read<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    value: Option<u64>,
) -> Result<Decision<S>, HandlerExecutionDenial> {
    let members = match reader
        .prior_output_family_if_present::<Preserve<S>, SurvivingNodes<S>>()
        .map_err(HandlerExecutionDenial::new)?
    {
        Some(m) => m.into_iter().map(|m| m.into_identity()).collect::<Vec<_>>(),
        None => reader
            .prior_output_family::<producer::Binding<S>, InitialNodes<S>>()
            .map_err(HandlerExecutionDenial::new)?
            .into_iter()
            .map(|m| m.into_identity())
            .collect(),
    };
    assert!(members.len() <= 1);
    let own = members.first();
    if !matches!(value, Some(JOIN_ORBIT | RETIRE_ANCHOR)) {
        return Ok(Decision::Keep(
            own.map(|own| reader.mutation_target(own)).transpose()?,
        ));
    }
    let foreign = reader.resolve_entity(Number::<S::Boundary>::at("Number"), 1)?;
    let join = value == Some(JOIN_ORBIT);
    if !join {
        let orbit = reader
            .orbit(Next::reference(), &foreign)
            .map_err(HandlerExecutionDenial::new)?;
        assert_eq!(orbit.len(), if own.is_some() { 2 } else { 1 });
        if let Some(own) = own {
            assert_eq!(
                orbit.member(0).unwrap().entity_id(),
                own.entity_id(),
                "the old walk root retires, while the selected foreign member survives"
            );
        }
    }
    let mut edges = Vec::new();
    if let Some(own) = own {
        for member in [own, &foreign] {
            for edge in reader
                .relations_from(Next::reference(), member)
                .map_err(HandlerExecutionDenial::new)?
            {
                edges.push((
                    reader.mutation_target(edge.from())?,
                    reader.mutation_target(edge.to())?,
                ));
            }
        }
    }
    Ok(Decision::Edit {
        own: own.map(|own| reader.mutation_target(own)).transpose()?,
        foreign: reader.mutation_target(&foreign)?,
        edges,
        join,
    })
}
pub fn trim<S: Scenario>(
    writer: &mut CandidateWriter<'_, S, Preserve<S>>,
    decision: Decision<S>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let key = "0";
    let (own, foreign, edges, join) = match decision {
        Decision::Keep(None) | Decision::Edit { own: None, .. } => return Ok(()),
        Decision::Keep(Some(own)) => {
            let own = writer.projected_entity(&own)?;
            return writer.preserve_member::<SurvivingNodes<S>>(key, &own);
        }
        Decision::Edit {
            own: Some(own),
            foreign,
            edges,
            join,
        } => (own, foreign, edges, join),
    };
    for (from, to) in edges {
        let from = writer.projected_entity(&from)?;
        let to = writer.projected_entity(&to)?;
        writer.unlink(Next::reference(), &from, &to)?;
    }
    let own = writer.projected_entity(&own)?;
    let foreign = writer.projected_entity(&foreign)?;
    if join {
        writer.link(Next::reference(), "mixed-forward", &own, &foreign)?;
        writer.link(Next::reference(), "mixed-reverse", &foreign, &own)?;
        writer.preserve_member::<SurvivingNodes<S>>(key, &own)?;
    } else {
        writer.delete_entity(Occurrence::<S::Boundary>::reference(), &own)?;
        writer.retire_member::<RetiredNodes<S>>(key, &own)?;
        writer.link(Next::reference(), "foreign-self-loop", &foreign, &foreign)?;
    }
    Ok(())
}
