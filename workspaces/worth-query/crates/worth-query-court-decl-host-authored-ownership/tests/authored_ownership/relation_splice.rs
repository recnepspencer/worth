//! Mid-thread authored splice and its independently checked exact relation inverse.
use super::{lifecycle::*, *};
mod checkpoint;
mod courts;
pub mod inspection;
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, worth_query_decl::facade::application_schema::WorthValue,
)]
pub struct Coordinate {
    partition: u32,
    slot: u64,
    generation: u32,
}
#[derive(Clone, Copy, Debug, worth_query_decl::facade::application_schema::WorthValue)]
pub struct Ends {
    from: Coordinate,
    to: Coordinate,
}
#[derive(Clone, Debug, worth_query_decl::facade::application_schema::WorthValue)]
pub struct Material {
    pub before: Option<Ends>,
    pub after: Option<Ends>,
}
pub struct Restoration<S: Scenario> {
    pub scope: Target<S>,
    pub relations: Vec<RelationTargets<S>>,
}
pub struct RelationTargets<S: Scenario> {
    remove: Option<(Target<S>, Target<S>)>,
    restore: Option<(Option<Target<S>>, Option<Target<S>>)>,
}
impl OperationLinks<RetireOp> for PreviousStage {}
impl OperationLinks<RestoreOp> for PreviousStage {}
impl OperationReads<RestoreOp> for PreviousStage {}
impl OperationUnlinks<RestoreOp> for PreviousStage {}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if matches!(S::MODE, 30 | 39 | 40 | 41) {
        inspection::declare(schema)
    } else {
        schema
    }
}
fn coordinate(id: EntityId) -> Coordinate {
    Coordinate {
        partition: id.partition_id.0,
        slot: id.local_slot.0,
        generation: id.generation.0,
    }
}
fn historical(id: Coordinate) -> EntityId {
    EntityId::new(PartitionId(id.partition), id.slot, id.generation)
}
pub fn derive<S: Scenario>(
    fields: &dyn ApplicationCorrectionPreImage<S>,
    case: u8,
) -> Result<(Option<Coordinate>, Vec<Material>), ApplicationCorrectionDerivationDenial> {
    if !matches!(S::MODE, 30 | 39 | 40 | 41) {
        return Ok((None, Vec::new()));
    }
    use ApplicationCorrectionDerivationDenial::MissingPreImage;
    let retired = fields
        .output_entity(&<Retired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR)
        .ok_or(MissingPreImage)?;
    let locus = DeclaredPreImageRelationLocus::from_relation(PreviousStage::reference());
    let count = fields.relation_count(&locus).ok_or(MissingPreImage)?;
    let mut rows = Vec::new();
    for index in 0..count {
        let material = fields.relation_at(&locus, index).ok_or(MissingPreImage)?;
        rows.push(Material {
            before: material.before().map(|(a, b)| Ends {
                from: coordinate(a),
                to: coordinate(b),
            }),
            after: material.after().map(|(a, b)| Ends {
                from: coordinate(a),
                to: coordinate(b),
            }),
        });
    }
    if case == 32 {
        let row = rows.iter_mut().find(|row| row.before.is_some()).unwrap();
        let Ends { from, to } = row.before.unwrap();
        row.before = Some(Ends { from: to, to: from });
    }
    if case == 33 {
        rows.remove(rows.iter().position(|row| row.before.is_some()).unwrap());
    }
    if case == 34 {
        let end = rows.iter().find_map(|row| row.after).unwrap().to;
        rows.push(Material {
            before: Some(Ends { from: end, to: end }),
            after: None,
        });
    }
    if case == 35 {
        rows.retain(|row| row.after.is_none());
    }
    if case == 36 {
        let Ends { from, to } = rows.iter().find_map(|row| row.after).unwrap();
        rows.push(Material {
            before: None,
            after: Some(Ends { from: to, to: from }),
        });
    }
    Ok((Some(coordinate(retired)), rows))
}
pub fn read_retirement<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, RetireBinding<S>>,
    stage: &WorthQueryInvariantEntityIdentity<S, Node>,
    removed: &mut Vec<(Target<S>, Target<S>)>,
    replacement: &mut Vec<(Target<S>, Target<S>)>,
    case: u8,
) -> Result<(), HandlerExecutionDenial> {
    if case == 38 {
        return Ok(());
    }
    let outgoing = reader
        .relations_from(PreviousStage::reference(), stage)
        .map_err(HandlerExecutionDenial::new)?;
    let incoming = reader
        .relations_to(PreviousStage::reference(), stage)
        .map_err(HandlerExecutionDenial::new)?;
    for edge in outgoing.iter().chain(&incoming) {
        if case == 41 {
            continue;
        }
        removed.push((
            reader.mutation_target(edge.from())?,
            reader.mutation_target(edge.to())?,
        ));
    }
    for next in &incoming {
        if case != 31 {
            reader
                .relations_from(PreviousStage::reference(), next.from())
                .map_err(HandlerExecutionDenial::new)?;
        }
        for previous in &outgoing {
            replacement.push((
                reader.mutation_target(next.from())?,
                reader.mutation_target(previous.to())?,
            ));
        }
    }
    Ok(())
}
pub fn read_restoration<S: Scenario>(
    input: &Restore,
    reader: &mut DecisionReader<'_, '_, '_, S, RestoreBinding<S>>,
) -> Result<Restoration<S>, HandlerExecutionDenial> {
    let scope = reader.resolve_entity(Id::reference(), input.scope)?;
    let scope = reader.mutation_target(&scope)?;
    let mut relations = Vec::new();
    for material in &input.relation_material {
        let remove = if let Some(Ends { from, to }) = material.after {
            let from = reader.resolve_historical_entity(Node::reference(), historical(from))?;
            let to = reader.resolve_historical_entity(Node::reference(), historical(to))?;
            reader
                .relations_from(PreviousStage::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            Some((reader.mutation_target(&from)?, reader.mutation_target(&to)?))
        } else {
            None
        };
        let mut endpoint = |id| {
            if Some(id) == input.retired_entity {
                return Ok(None);
            }
            let identity = reader.resolve_historical_entity(Node::reference(), historical(id))?;
            reader.mutation_target(&identity).map(Some)
        };
        let restore = match material.before {
            Some(Ends { from, to }) => Some((endpoint(from)?, endpoint(to)?)),
            None => None,
        };
        relations.push(RelationTargets { remove, restore });
    }
    Ok(Restoration { scope, relations })
}
pub fn write_restoration<S: Scenario>(
    relations: Vec<RelationTargets<S>>,
    created: &WorthQueryApplicationEffectEntity<S, Node>,
    writer: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for (index, relation) in relations.into_iter().enumerate() {
        if let Some((from, to)) = relation.remove {
            let from = writer.projected_entity(&from)?;
            let to = writer.projected_entity(&to)?;
            writer.unlink(PreviousStage::reference(), &from, &to)?;
        }
        if let Some((from, to)) = relation.restore {
            let from = from
                .map(|target| writer.projected_entity(&target))
                .transpose()?;
            let to = to
                .map(|target| writer.projected_entity(&target))
                .transpose()?;
            writer.link(
                PreviousStage::reference(),
                &format!("restore-splice-{index}"),
                from.as_ref().unwrap_or(created),
                to.as_ref().unwrap_or(created),
            )?;
        }
    }
    Ok(())
}
