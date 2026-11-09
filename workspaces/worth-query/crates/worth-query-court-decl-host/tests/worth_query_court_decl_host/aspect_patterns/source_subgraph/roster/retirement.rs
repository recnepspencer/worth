//! Native independently compares these submitted roles to live owner adjacency.
use super::*;
pub struct Handler {
    pub extra_member: bool,
}
pub fn observe<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    input: &Retire,
) -> Result<Retirement, HandlerExecutionDenial>
where
    Source: OperationReads<B::Operation>,
    Member: OperationReads<B::Operation>,
    OwnedBy: OperationReads<B::Operation>,
    Next: OperationReads<B::Operation>,
{
    let root = reader.resolve_entity(SourceKey::at("SourceKey"), input.key.clone())?;
    reader.entity(&root, Source::reference())?;
    reader.field(&root, SourceKey::at("SourceKey"))?;
    reader.field(&root, SourceValue::at("SourceValue"))?;
    reader
        .relations_to(OwnedBy::reference(), &root)
        .map_err(HandlerExecutionDenial::new)?;
    let mut members = Vec::new();
    let mut links = Vec::new();
    let mut observed = std::collections::BTreeSet::new();
    for key in &input.members {
        let member = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
        if observed.insert(key) {
            reader.entity(&member, Member::reference())?;
            reader.field(&member, MemberKey::at("MemberKey"))?;
            reader.field(&member, Value::at("MemberValue"))?;
            reader
                .relations_from(OwnedBy::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?;
            for edge in reader
                .relations_from(Next::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?
            {
                links.push((
                    reader.mutation_target(edge.from())?,
                    reader.mutation_target(edge.to())?,
                ));
            }
        }
        members.push((key.clone(), reader.mutation_target(&member)?));
    }
    Ok(Retirement {
        scope: creation::scope(reader)?,
        root: reader.mutation_target(&root)?,
        members,
        links,
    })
}
pub fn graph<B, Root, Members>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    decision: Retirement,
) -> Result<AnchorTarget, WorthQueryApplicationAttemptDenial>
where
    B: ApplicationMutationBinding<Schema>,
    Source: OperationDeletes<B::Operation>,
    Member: OperationDeletes<B::Operation>,
    OwnedBy: OperationReads<B::Operation> + OperationUnlinks<B::Operation>,
    Next: OperationReads<B::Operation> + OperationUnlinks<B::Operation>,
    Root: WorthQueryApplicationOutputRole<
        Schema = Schema,
        Contract = B::Output,
        Entity = Source,
        Action = WorthQueryRetireOutput,
    >,
    Members: WorthQueryApplicationOutputRoleFamily<
        Schema = Schema,
        Contract = B::Output,
        Entity = Member,
    >,
{
    let root = writer.projected_entity(&decision.root)?;
    for (from, to) in decision.links {
        let from = writer.projected_entity(&from)?;
        let to = writer.projected_entity(&to)?;
        writer.unlink(Next::reference(), &from, &to)?;
    }
    let mut seen = std::collections::BTreeSet::new();
    for (index, (key, target)) in decision.members.into_iter().enumerate() {
        let member = writer.projected_entity(&target)?;
        if seen.insert(key.clone()) {
            writer.unlink(OwnedBy::reference(), &member, &root)?;
            writer.delete_entity(Member::reference(), &member)?;
            writer.retire_member::<Members>(&key, &member)?;
        } else {
            writer.retire_member::<Members>(&format!("repeated-{index}:{key}"), &member)?;
        }
    }
    writer.delete_entity(Source::reference(), &root)?;
    writer.retire_output::<Root>(&root)?;
    Ok(decision.scope)
}
impl OperationHandler<Schema, RetireBinding> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, Schema, RetireBinding>,
    ) -> HandlerResult<Retirement, edit::Denial> {
        if input
            .replacement
            .as_ref()
            .is_some_and(|image| !creation::valid(image))
        {
            return HandlerResult::DomainDenied(edit::Denial::InvalidSourceGraph);
        }
        match observe(reader, input) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Retire,
        target: Retirement,
        writer: &mut CandidateWriter<'_, Schema, RetireBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let scope = graph::<_, Retired, RetiredMembers>(writer, target)?;
            if self.extra_member {
                let parent = writer.projected_entity(&scope)?;
                let extra = writer.create_entity_in_context(
                    &parent,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new("extra-member").unwrap(),
                )?;
                writer.initialize_field(
                    &extra,
                    MemberKey::at("MemberKey"),
                    "extra-member".to_owned(),
                )?;
                writer.initialize_field(&extra, Value::at("MemberValue"), 0)?;
                writer.delete_entity(Member::reference(), &extra)?;
                writer.retire_member::<RetiredMembers>("extra-member", &extra)?;
            }

            if let Some(image) = &input.replacement {
                creation::graph::<_, ReplacementCreated, ReplacementMembers>(
                    writer, &scope, image,
                )?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
