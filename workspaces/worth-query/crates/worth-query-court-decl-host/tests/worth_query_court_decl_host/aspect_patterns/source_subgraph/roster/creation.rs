//! Both ordinary creation and recorded restoration use native field/entity/relation effects.
use super::*;
pub struct Handler;
macro_rules! creation {
    ($binding:ty, $input:ty) => {
        impl OperationHandler<Schema, $binding> for Handler {
            fn decide(
                &self,
                input: &$input,
                reader: &mut DecisionReader<'_, '_, '_, Schema, $binding>,
            ) -> HandlerResult<AnchorTarget, edit::Denial> {
                if input.0.members.iter().any(|row| {
                    row.next.as_ref().is_some_and(|next| {
                        !input
                            .0
                            .members
                            .iter()
                            .any(|candidate| &candidate.key == next)
                    })
                }) {
                    return HandlerResult::DomainDenied(edit::Denial::InvalidSourceGraph);
                }
                let result = reader
                    .resolve_entity(SourceKey::at("SourceKey"), "source".to_owned())
                    .and_then(|root| reader.mutation_target(&root));
                match result {
                    Ok(value) => HandlerResult::Completed(value),
                    Err(error) => HandlerResult::ExecutionDenied(error),
                }
            }
            fn build_candidate(
                &self,
                input: &$input,
                scope: AnchorTarget,
                writer: &mut CandidateWriter<'_, Schema, $binding>,
            ) -> HandlerResult<edit::Edited, edit::Denial> {
                let result = (|| {
                    let scope = writer.projected_entity(&scope)?;
                    let root = writer.create_entity_in_context(
                        &scope,
                        Source::reference(),
                        WorthQueryApplicationEntityKey::new(&input.0.key).unwrap(),
                    )?;
                    writer.initialize_field(
                        &root,
                        SourceKey::at("SourceKey"),
                        input.0.key.clone(),
                    )?;
                    writer.initialize_field(
                        &root,
                        SourceValue::at("SourceValue"),
                        input.0.value,
                    )?;
                    writer.create_output::<Created>(&root)?;
                    let mut members = Vec::new();
                    for row in &input.0.members {
                        let member = writer.create_entity_in_context(
                            &scope,
                            Member::reference(),
                            WorthQueryApplicationEntityKey::new(&row.key).unwrap(),
                        )?;
                        writer.initialize_field(
                            &member,
                            MemberKey::at("MemberKey"),
                            row.key.clone(),
                        )?;
                        writer.initialize_field(&member, Value::at("MemberValue"), row.value)?;
                        writer.link(
                            OwnedBy::reference(),
                            format!("owner:{}", row.key),
                            &member,
                            &root,
                        )?;
                        writer.create_member::<CreatedMembers>(&row.key, &member)?;
                        members.push((row.key.clone(), member));
                    }
                    for row in &input.0.members {
                        if let Some(next) = &row.next {
                            let from = &members.iter().find(|(key, _)| key == &row.key).unwrap().1;
                            let to = &members.iter().find(|(key, _)| key == next).unwrap().1;
                            writer.link(
                                Next::reference(),
                                format!("next:{}", row.key),
                                from,
                                to,
                            )?;
                        }
                    }
                    Ok::<_, WorthQueryApplicationAttemptDenial>(())
                })();
                match result {
                    Ok(()) => HandlerResult::Completed(edit::Edited),
                    Err(error) => {
                        HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                    }
                }
            }
        }
    };
}
creation!(CreateBinding, Create);
creation!(RestoreBinding, Restore);
