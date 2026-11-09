//! The caller submits a roster; Native compares it to owner adjacency independently.
use super::*;
pub struct Handler;
impl OperationHandler<Schema, RetireBinding> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, Schema, RetireBinding>,
    ) -> HandlerResult<Retirement, edit::Denial> {
        let result = (|| {
            let root = reader.resolve_entity(SourceKey::at("SourceKey"), input.key.clone())?;
            reader.entity(&root, Source::reference())?;
            reader.field(&root, SourceKey::at("SourceKey"))?;
            reader.field(&root, SourceValue::at("SourceValue"))?;
            reader
                .relations_to(OwnedBy::reference(), &root)
                .map_err(HandlerExecutionDenial::new)?;
            let mut members = Vec::new();
            let mut links = Vec::new();
            for key in &input.members {
                let member = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
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
                members.push((key.clone(), reader.mutation_target(&member)?));
            }
            Ok(Retirement {
                root: reader.mutation_target(&root)?,
                members,
                links,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Retire,
        decision: Retirement,
        writer: &mut CandidateWriter<'_, Schema, RetireBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let root = writer.projected_entity(&decision.root)?;
            for (from, to) in decision.links {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.unlink(Next::reference(), &from, &to)?;
            }
            for (key, target) in decision.members {
                let member = writer.projected_entity(&target)?;
                writer.unlink(OwnedBy::reference(), &member, &root)?;
                writer.delete_entity(Member::reference(), &member)?;
                writer.retire_member::<RetiredMembers>(&key, &member)?;
            }
            writer.delete_entity(Source::reference(), &root)?;
            writer.retire_output::<Retired>(&root)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
