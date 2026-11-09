//! Ordinary creation and recorded restoration use the same native graph effects.
use super::*;
pub struct Handler;
pub struct RestorationHandler {
    pub omit_anchor: bool,
}
pub fn valid(image: &Image) -> bool {
    image.members.iter().all(|row| {
        row.next
            .as_ref()
            .is_none_or(|next| image.members.iter().any(|candidate| &candidate.key == next))
    })
}
pub fn scope<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
) -> Result<AnchorTarget, HandlerExecutionDenial> {
    let scope = reader.resolve_entity(SourceKey::at("SourceKey"), "source".to_owned())?;
    reader.mutation_target(&scope)
}
pub fn graph<B, Root, Members>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    scope: &AnchorTarget,
    image: &Image,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    B: ApplicationMutationBinding<Schema>,
    Source: OperationCreates<B::Operation>,
    Member: OperationCreates<B::Operation>,
    OwnedBy: OperationLinks<B::Operation>,
    Next: OperationLinks<B::Operation>,
    Root: WorthQueryApplicationOutputRole<
        Schema = Schema,
        Contract = B::Output,
        Entity = Source,
        Action = WorthQueryCreateOutput,
    >,
    Members: WorthQueryApplicationOutputRoleFamily<
        Schema = Schema,
        Contract = B::Output,
        Entity = Member,
    >,
{
    let scope = writer.projected_entity(scope)?;
    let root = writer.create_entity_in_context(
        &scope,
        Source::reference(),
        WorthQueryApplicationEntityKey::new(&image.key).unwrap(),
    )?;
    writer.initialize_field(&root, SourceKey::at("SourceKey"), image.key.clone())?;
    writer.initialize_field(&root, SourceValue::at("SourceValue"), image.value)?;
    writer.create_output::<Root>(&root)?;
    let mut members = Vec::new();
    for row in &image.members {
        let member = writer.create_entity_in_context(
            &scope,
            Member::reference(),
            WorthQueryApplicationEntityKey::new(&row.key).unwrap(),
        )?;
        writer.initialize_field(&member, MemberKey::at("MemberKey"), row.key.clone())?;
        writer.initialize_field(&member, Value::at("MemberValue"), row.value)?;
        writer.link(
            OwnedBy::reference(),
            format!("owner:{}", row.key),
            &member,
            &root,
        )?;
        writer.create_member::<Members>(&row.key, &member)?;
        members.push((row.key.clone(), member));
    }
    for row in &image.members {
        if let Some(next) = &row.next {
            let from = &members.iter().find(|(key, _)| key == &row.key).unwrap().1;
            let to = &members.iter().find(|(key, _)| key == next).unwrap().1;
            writer.link(Next::reference(), format!("next:{}", row.key), from, to)?;
        }
    }
    Ok(())
}
impl OperationHandler<Schema, CreateBinding> for Handler {
    fn decide(
        &self,
        input: &Create,
        reader: &mut DecisionReader<'_, '_, '_, Schema, CreateBinding>,
    ) -> HandlerResult<AnchorTarget, edit::Denial> {
        if !valid(&input.0) {
            return HandlerResult::DomainDenied(edit::Denial::InvalidSourceGraph);
        }
        match scope(reader) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Create,
        target: AnchorTarget,
        writer: &mut CandidateWriter<'_, Schema, CreateBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        match graph::<_, Created, CreatedMembers>(writer, &target, &input.0) {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl OperationHandler<Schema, RestoreBinding> for RestorationHandler {
    fn decide(
        &self,
        input: &Restore,
        reader: &mut DecisionReader<'_, '_, '_, Schema, RestoreBinding>,
    ) -> HandlerResult<Restoration, edit::Denial> {
        if !valid(&input.image) {
            return HandlerResult::DomainDenied(edit::Denial::InvalidSourceGraph);
        }
        let result = (|| {
            Ok(Restoration {
                scope: scope(reader)?,
                remove: input
                    .remove
                    .as_ref()
                    .map(|remove| retirement::observe(reader, remove))
                    .transpose()?,
            })
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Restore,
        target: Restoration,
        writer: &mut CandidateWriter<'_, Schema, RestoreBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            if let Some(remove) = target.remove {
                retirement::graph::<_, ReplacementRetired, RemovedMembers>(writer, remove, true)?;
            }
            if self.omit_anchor {
                let scope = writer.projected_entity(&target.scope)?;
                for row in &input.image.members {
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
                    writer.create_member::<RestoredMembers>(&row.key, &member)?;
                }
                Ok(())
            } else {
                graph::<_, Restored, RestoredMembers>(writer, &target.scope, &input.image)
            }
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
