//! Actual public handler contacts and candidate-aware stage lookup.
use super::*;
pub struct Provider;
thread_local! { static CONTACTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(in super::super::super) fn contacts() -> usize {
    CONTACTS.with(|count| count.get())
}
pub(in super::super::super) fn reset_contacts() {
    CONTACTS.with(|count| count.set(0));
}
impl<S: Scenario> OperationHandler<S, Binding<S>> for Provider {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, S, Binding<S>>,
    ) -> HandlerResult<Decision<S>, ()> {
        CONTACTS.with(|count| count.set(count.get() + 1));
        let result = (|| {
            let source = reader.resolve_entity(Id::reference(), input.id)?;
            let body = match reader.prior_output_if_present::<Binding<S>, Body<S>>().map_err(HandlerExecutionDenial::new)? {
                Some(body) => body,
                None => reader.prior_output::<super::super::super::producer::Binding<S>, super::super::super::producer::Role<S>>().map_err(HandlerExecutionDenial::new)?,
            };
            let cell = match reader.prior_output_if_present::<Binding<S>, Cell<S>>().map_err(HandlerExecutionDenial::new)? {
                Some(cell) => cell,
                None => reader.prior_output::<super::super::super::producer::Binding<S>, super::super::super::producer::Cell<S>>().map_err(HandlerExecutionDenial::new)?,
            };
            let mut next_cell = reader
                .field(&cell, Id::reference())?
                .and_then(|id| id.checked_add(1))
                .ok_or_else(|| {
                    HandlerExecutionDenial::new(std::io::Error::other(
                        "prior cell key is absent or exhausted",
                    ))
                })?;
            if S::MODE == 9 && input.id == 11 {
                next_cell = 102;
            }
            let cell_value = if S::MODE == 43 {
                let source_value = super::super::super::content::read(reader, &source)?;
                assert_eq!(
                    source_value,
                    input.id * 100,
                    "actual candidate source content"
                );
                for entity in [&body, &cell] {
                    let word = super::super::super::content::read(reader, entity)?;
                    assert_eq!(
                        Some(word),
                        reader.field(entity, Value::reference())?,
                        "the immutable field's content must match its ordinary value"
                    );
                }
                let cell_id = reader.field(&cell, Id::reference())?.unwrap();
                if cell_id != input.id * 10 + 2 {
                    assert_eq!(
                        super::super::super::content::read(reader, &cell)?,
                        source_value + cell_id,
                        "read the prior writer's actual committed bytes"
                    );
                }
                source_value + next_cell
            } else if S::MODE == 7 && input.id == 10 {
                let peer = reader.resolve_entity(Id::reference(), 111)?;
                reader.field(&peer, Value::reference())?.unwrap_or(46)
            } else {
                46
            };
            reader.field(&body, Value::reference())?;
            let mut previous_edges = Vec::new();
            for edge in reader
                .relations_from(Peer::reference(), &cell)
                .map_err(HandlerExecutionDenial::new)?
            {
                previous_edges.push((
                    reader.mutation_target(edge.from())?,
                    reader.mutation_target(edge.to())?,
                ));
            }
            let mut own_edges = Vec::new();
            if matches!(S::MODE, 64) && input.id == 10 {
                let foreign = reader.resolve_entity(Id::reference(), 111)?;
                let walked = reader
                    .walk(OwnCell::reference(), RelationDirection::Reverse, &foreign)
                    .map_err(HandlerExecutionDenial::new)?;
                if walked.len()
                    != (if S::MODE == 64 && next_cell == 104 {
                        3
                    } else {
                        2
                    })
                {
                    return Err(HandlerExecutionDenial::new(std::io::Error::other(
                        "the actual foreign walk is incomplete",
                    )));
                }
            }

            if matches!(S::MODE, 42 | 64) {
                let walk = reader
                    .walk(OwnCell::reference(), RelationDirection::Reverse, &body)
                    .map_err(HandlerExecutionDenial::new)?;
                if walk.len() != 2 {
                    return Err(HandlerExecutionDenial::new(std::io::Error::other(
                        "actual writer predecessor walk is incomplete",
                    )));
                }
            }

            if matches!(S::MODE, 22 | 23 | 26 | 27 | 28 | 42 | 64) {
                if S::MODE == 22 || (matches!(S::MODE, 27 | 28) && input.id == 10) {
                    let component = reader
                        .component_of(Components::reference(), &body)
                        .map_err(HandlerExecutionDenial::new)?;
                    if component.len() != 2 {
                        return Err(HandlerExecutionDenial::new(std::io::Error::other(
                            "body predecessor component is incomplete",
                        )));
                    }
                } else if S::MODE == 26 {
                    let member = reader
                        .entity_member(&body)
                        .map_err(HandlerExecutionDenial::new)?;
                    let component = reader
                        .graph_component_of(
                            Components::reference(),
                            DerivedCell::reference(),
                            &member,
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    if component.len() != 2 {
                        return Err(HandlerExecutionDenial::new(std::io::Error::other(
                            "derived predecessor component is incomplete",
                        )));
                    }
                } else if S::MODE == 23 {
                    let cells = reader
                        .derived_relations_from(DerivedCell::reference(), &body)
                        .map_err(HandlerExecutionDenial::new)?;
                    if cells.len() != 1
                        || reader.field(cells[0].to(), Id::reference())?
                            != reader.field(&cell, Id::reference())?
                    {
                        return Err(HandlerExecutionDenial::new(std::io::Error::other(
                            "derived predecessor cell is incomplete",
                        )));
                    }
                }
                for edge in reader
                    .relations_from(OwnCell::reference(), &cell)
                    .map_err(HandlerExecutionDenial::new)?
                {
                    own_edges.push((
                        reader.mutation_target(edge.from())?,
                        reader.mutation_target(edge.to())?,
                    ));
                }
            }
            if S::MODE == 40 {
                for edge in reader
                    .relations_from(PreviousStage::reference(), &cell)
                    .map_err(HandlerExecutionDenial::new)?
                {
                    own_edges.push((
                        reader.mutation_target(edge.from())?,
                        reader.mutation_target(edge.to())?,
                    ));
                }
            }
            let selected_stage = reader
                .resolve_optional_entity(Id::reference(), if S::MODE == 15 { 101 } else { 20 })?;
            let selected_stage =
                if matches!(S::MODE, 30 | 40 | 41 | 43 | 61) && selected_stage.is_none() {
                    reader.resolve_optional_entity(Id::reference(), 30)?
                } else {
                    selected_stage
                };
            if S::MODE == 43 {
                if let Some(stage) = &selected_stage {
                    assert_eq!(
                        super::super::super::content::read(reader, stage)?,
                        reader.field(stage, Id::reference())?.unwrap() * 100,
                        "a freshly restored stage supplies its actual candidate bytes"
                    );
                }
                if input.id == 11 {
                    let preceding_writer = reader.resolve_entity(Id::reference(), 101)?;
                    assert_eq!(
                        super::super::super::content::read(reader, &preceding_writer)?,
                        u64::from(selected_stage.is_some()),
                        "the second writer reads the first writer's unpublished content"
                    );
                }
            }
            let foreign_member = if S::MODE == 61
                && input.id == 10
                && selected_stage
                    .as_ref()
                    .map(|stage| reader.field(stage, Id::reference()))
                    .transpose()?
                    .flatten()
                    == Some(30)
            {
                let foreign = reader.resolve_entity(Id::reference(), 113)?;
                reader.entity(&foreign, Node::reference())?;
                Some(reader.mutation_target(&foreign)?)
            } else {
                None
            };
            let selected_stage_id = selected_stage.as_ref().map(|stage| stage.entity_id());
            let stage = selected_stage
                .map(|stage| reader.mutation_target(&stage))
                .transpose()?;
            if S::MODE == 21 {
                let surviving_stage = reader.resolve_entity(Id::reference(), 30)?;
                let component = reader
                    .component_of(Components::reference(), &surviving_stage)
                    .map_err(HandlerExecutionDenial::new)?;
                let expected = if stage.is_some() { 2 } else { 1 };
                if component.len() != expected {
                    return Err(HandlerExecutionDenial::new(std::io::Error::other(
                        "coupled stage component did not observe the candidate deletion",
                    )));
                }
            }
            let other_stage = if matches!(S::MODE, 60 | 61 | 64 | 30 | 40 | 41) && stage.is_some() {
                reader
                    .resolve_optional_entity(Id::reference(), 30)?
                    .filter(|other| selected_stage_id != Some(other.entity_id()))
                    .map(|other| reader.mutation_target(&other))
                    .transpose()?
            } else {
                None
            };
            let visibility_target = if S::MODE == 64 && input.id == 10 && next_cell == 103 {
                let foreign_body = reader.resolve_entity(Id::reference(), 111)?;
                Some(reader.mutation_target(&foreign_body)?)
            } else if S::MODE == 28 && input.id == 11 {
                let foreign_body = reader.resolve_entity(Id::reference(), 10)?;
                Some(reader.mutation_target(&foreign_body)?)
            } else {
                None
            };
            Ok::<_, HandlerExecutionDenial>(Decision {
                boundary: <S::Boundary as sheet_boundary::Profile>::read(reader)?,
                next_cell,
                cell_value,
                source: reader.mutation_target(&source)?,
                body: reader.mutation_target(&body)?,
                previous_cell: reader.mutation_target(&cell)?,
                previous_edges,
                own_edges,
                stage,
                visibility_target,
                other_stage,
                foreign_member,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _input: &Input,
        decision: Decision<S>,
        writer: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            if let Some(foreign) = decision.foreign_member {
                let foreign = writer.projected_entity(&foreign)?;
                writer.delete_entity(Node::reference(), &foreign)?;
            }
            for (from, to) in decision.previous_edges {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.unlink(Peer::reference(), &from, &to)?;
            }
            for (from, to) in decision.own_edges {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                if S::MODE == 40 {
                    writer.unlink(PreviousStage::reference(), &from, &to)?;
                } else {
                    writer.unlink(OwnCell::reference(), &from, &to)?;
                }
            }
            let previous = writer.projected_entity(&decision.previous_cell)?;
            writer.delete_entity(Node::reference(), &previous)?;
            writer.retire_output::<RetiredCell<S>>(&previous)?;
            let body = writer.projected_entity(&decision.body)?;
            writer.write_field(
                &body,
                Value::reference(),
                if decision.stage.is_some() { 1 } else { 0 },
            )?;
            if S::MODE == 43 {
                writer
                    .write_content(
                        &body,
                        super::super::super::content::field(),
                        &(if decision.stage.is_some() { 1 } else { 0 }),
                    )
                    .expect("valid installed word content");
            }
            writer.preserve_output::<Body<S>>(&body)?;
            let source = writer.projected_entity(&decision.source)?;
            <S::Boundary as sheet_boundary::Profile>::trim(writer, decision.boundary, &source)?;
            let cell = writer.create_entity_in_context(
                &source,
                Node::reference(),
                WorthQueryApplicationEntityKey::new("body-cell-successor").unwrap(),
            )?;
            writer.initialize_field(&cell, Id::reference(), decision.next_cell)?;
            writer.initialize_field(&cell, Value::reference(), decision.cell_value)?;
            if S::MODE == 43 {
                writer
                    .initialize_content(
                        &cell,
                        super::super::super::content::field(),
                        &decision.cell_value,
                    )
                    .expect("valid installed word content");
            }
            writer.create_output::<Cell<S>>(&cell)?;
            if matches!(S::MODE, 22 | 23 | 26 | 27 | 28 | 42 | 64) {
                writer.link(OwnCell::reference(), "owned-cell-successor", &cell, &body)?;
            }
            if S::MODE == 40 {
                writer.link(
                    PreviousStage::reference(),
                    "owned-stage-successor",
                    &cell,
                    &body,
                )?;
            }
            if let Some(target) = decision.visibility_target {
                let target = writer.projected_entity(&target)?;
                if S::MODE == 64 {
                    writer.link(
                        OwnCell::reference(),
                        "foreign-visibility-boundary",
                        &cell,
                        &target,
                    )?;
                } else {
                    writer.link(
                        StageVisible::reference(),
                        "foreign-visibility-boundary",
                        &cell,
                        &target,
                    )?;
                }
            }
            if let Some(other) = decision.other_stage {
                let other = writer.projected_entity(&other)?;
                writer.link(
                    Peer::reference(),
                    "body-other-stage-successor",
                    &cell,
                    &other,
                )?;
            }
            if let Some(stage) = decision.stage {
                let stage = writer.projected_entity(&stage)?;
                writer.link(Peer::reference(), "body-stage-successor", &cell, &stage)?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
