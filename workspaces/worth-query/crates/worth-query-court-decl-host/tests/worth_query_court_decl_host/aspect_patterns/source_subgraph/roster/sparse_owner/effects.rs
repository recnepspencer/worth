//! The revision reads exactly what it changes and writes only that delta.
use super::*;
pub struct Handler;
impl OperationHandler<Schema, ReviseBinding> for Handler {
    fn decide(
        &self,
        input: &Revise,
        reader: &mut DecisionReader<'_, '_, '_, Schema, ReviseBinding>,
    ) -> HandlerResult<Revised, edit::Denial> {
        let result = (|| {
            let owner = reader.resolve_entity(SourceKey::at("SourceKey"), input.owner.clone())?;
            if input.value.is_some() {
                reader.field(&owner, SourceValue::at("SourceValue"))?;
            }
            // Reading the mark observes it present or absent on the owner.
            if input.mark != MarkEdit::Unchanged {
                reader.field(&owner, OwnerMark::at("OwnerMark"))?;
            }
            let mut leaving = Vec::new();
            for key in &input.retire {
                let member = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                reader.entity(&member, Member::reference())?;
                reader.field(&member, MemberKey::at("MemberKey"))?;
                reader.field(&member, Value::at("MemberValue"))?;
                reader
                    .relations_from(OwnedBy::reference(), &member)
                    .map_err(HandlerExecutionDenial::new)?;
                leaving.push(reader.mutation_target(&member)?);
            }
            Ok(Revised {
                owner: reader.mutation_target(&owner)?,
                leaving,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Revise,
        target: Revised,
        writer: &mut CandidateWriter<'_, Schema, ReviseBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let owner = writer.projected_entity(&target.owner)?;
            if let Some(value) = input.value {
                writer.write_field(&owner, SourceValue::at("SourceValue"), value)?;
            }
            match input.mark {
                MarkEdit::Unchanged => (),
                MarkEdit::Clear => writer.remove_aspect(&owner, SourceMark::reference())?,
                MarkEdit::Set(mark) => {
                    writer.write_field(&owner, OwnerMark::at("OwnerMark"), mark)?
                }
            }
            for (key, member) in input.retire.iter().zip(&target.leaving) {
                let member = writer.projected_entity(member)?;
                writer.unlink(OwnedBy::reference(), &member, &owner)?;
                writer.delete_entity(Member::reference(), &member)?;
                writer.retire_member::<LeavingMembers>(key, &member)?;
            }
            for row in &input.create {
                let member = writer.create_entity_in_context(
                    &owner,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new(format!("arriving-member.{}", row.key))
                        .unwrap(),
                )?;
                writer.initialize_field(&member, MemberKey::at("MemberKey"), row.key.clone())?;
                writer.initialize_field(&member, Value::at("MemberValue"), row.value)?;
                writer.link(
                    OwnedBy::reference(),
                    format!("arriving-owner.{}", row.key),
                    &member,
                    &owner,
                )?;
                writer.create_member::<ArrivingMembers>(&row.key, &member)?;
            }
            writer.preserve_output::<RevisedOwner>(&owner)?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
