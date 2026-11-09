//! Ordinary readers observe every removed link; writers author only that local delta.
use super::*;
pub struct Handler(pub world::Fault);
impl OperationHandler<Schema, DeleteBinding> for Handler {
    fn decide(
        &self,
        input: &Delete,
        reader: &mut DecisionReader<'_, '_, '_, Schema, DeleteBinding>,
    ) -> HandlerResult<Selection, edit::Denial> {
        let result = (|| {
            let owner = reader.resolve_entity(SourceKey::at("SourceKey"), input.owner.clone())?;
            let member = reader.resolve_entity(MemberKey::at("MemberKey"), input.key.clone())?;
            let operand =
                reader.resolve_entity(MemberKey::at("MemberKey"), input.operand.clone())?;
            reader.entity(&member, Member::reference())?;
            reader.field(&member, MemberKey::at("MemberKey"))?;
            reader.field(&member, Value::at("MemberValue"))?;
            reader
                .relations_from(OwnedBy::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?;
            reader
                .relations_from(Head::reference(), &owner)
                .map_err(HandlerExecutionDenial::new)?;
            reader
                .relations_from(Operand::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Selection {
                owner: reader.mutation_target(&owner)?,
                member: reader.mutation_target(&member)?,
                operand: reader.mutation_target(&operand)?,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Delete,
        target: Selection,
        writer: &mut CandidateWriter<'_, Schema, DeleteBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let owner = writer.projected_entity(&target.owner)?;
            let member = writer.projected_entity(&target.member)?;
            let operand = writer.projected_entity(&target.operand)?;
            writer.unlink(OwnedBy::reference(), &member, &owner)?;
            writer.unlink(Head::reference(), &owner, &member)?;
            writer.unlink(Operand::reference(), &member, &operand)?;
            writer.delete_entity(Member::reference(), &member)?;
            writer.retire_output::<Deleted>(&member)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl OperationHandler<Schema, RestoreBinding> for Handler {
    fn decide(
        &self,
        input: &Restore,
        reader: &mut DecisionReader<'_, '_, '_, Schema, RestoreBinding>,
    ) -> HandlerResult<Restoration, edit::Denial> {
        let result = (|| {
            let owner_key = if self.0 == world::Fault::ForeignRestoredMember {
                "source"
            } else {
                &input.owner
            };
            let owner = reader.resolve_entity(SourceKey::at("SourceKey"), owner_key.to_owned())?;
            let operand =
                reader.resolve_entity(MemberKey::at("MemberKey"), input.operand.clone())?;
            Ok(Restoration {
                owner: reader.mutation_target(&owner)?,
                operand: reader.mutation_target(&operand)?,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
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
            let owner = writer.projected_entity(&target.owner)?;
            let operand = writer.projected_entity(&target.operand)?;
            let member = writer.create_entity_in_context(
                &owner,
                Member::reference(),
                WorthQueryApplicationEntityKey::new("returned-member").unwrap(),
            )?;
            let key = if self.0 == world::Fault::ChangedRestoredKey {
                "changed".into()
            } else {
                input.key.clone()
            };
            writer.initialize_field(&member, MemberKey::at("MemberKey"), key)?;
            writer.initialize_field(
                &member,
                Value::at("MemberValue"),
                input.value + u64::from(self.0 == world::Fault::ChangedRestoredValue),
            )?;
            writer.link(OwnedBy::reference(), "returned-owner", &member, &owner)?;
            writer.link(Head::reference(), "returned-head", &owner, &member)?;
            writer.link(Operand::reference(), "returned-operand", &member, &operand)?;
            writer.create_output::<Restored>(&member)?;
            if matches!(
                self.0,
                world::Fault::RepeatedRestoredMember | world::Fault::ExtraRestoredMember
            ) {
                let extra = writer.create_entity_in_context(
                    &owner,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new("additional-member").unwrap(),
                )?;
                let key = if self.0 == world::Fault::RepeatedRestoredMember {
                    input.key.clone()
                } else {
                    input.operand.clone()
                };
                writer.initialize_field(&extra, MemberKey::at("MemberKey"), key)?;
                writer.initialize_field(&extra, Value::at("MemberValue"), input.value)?;
                writer.link(OwnedBy::reference(), "additional-owner", &extra, &owner)?;
                writer.create_member::<Additional>("extra", &extra)?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
