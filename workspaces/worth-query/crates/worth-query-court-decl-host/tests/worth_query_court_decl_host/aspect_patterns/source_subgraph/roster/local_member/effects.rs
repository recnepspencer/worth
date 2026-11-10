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
            reader
                .relations_from(Head::reference(), &owner)
                .map_err(HandlerExecutionDenial::new)?;
            let mut members = Vec::new();
            for key in &input.members {
                let member = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                reader.entity(&member, Member::reference())?;
                reader.field(&member, MemberKey::at("MemberKey"))?;
                reader.field(&member, Value::at("MemberValue"))?;
                reader
                    .relations_from(OwnedBy::reference(), &member)
                    .map_err(HandlerExecutionDenial::new)?;
                reader
                    .relations_from(Operand::reference(), &member)
                    .map_err(HandlerExecutionDenial::new)?;
                members.push(reader.mutation_target(&member)?);
            }
            let operand = match &input.operand {
                Some(key) => {
                    let operand = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                    Some(reader.mutation_target(&operand)?)
                }
                None => None,
            };
            let mut notes = Vec::new();
            for key in &input.notes {
                let note = reader.resolve_entity(NoteKey::at("NoteKey"), key.clone())?;
                reader.entity(&note, Note::reference())?;
                reader.field(&note, NoteKey::at("NoteKey"))?;
                reader.field(&note, NoteValue::at("NoteValue"))?;
                reader
                    .relations_from(NotedBy::reference(), &note)
                    .map_err(HandlerExecutionDenial::new)?;
                notes.push(reader.mutation_target(&note)?);
            }
            let revalued = match &input.revalue {
                Some(row) => {
                    let member =
                        reader.resolve_entity(MemberKey::at("MemberKey"), row.key.clone())?;
                    reader.field(&member, Value::at("MemberValue"))?;
                    Some(reader.mutation_target(&member)?)
                }
                None => None,
            };
            Ok(Selection {
                owner: reader.mutation_target(&owner)?,
                members,
                operand,
                notes,
                revalued,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Delete,
        target: Selection,
        writer: &mut CandidateWriter<'_, Schema, DeleteBinding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let owner = writer.projected_entity(&target.owner)?;
            for (index, (key, member)) in input.members.iter().zip(&target.members).enumerate() {
                let member = writer.projected_entity(member)?;
                writer.unlink(OwnedBy::reference(), &member, &owner)?;
                if let (0, Some(operand)) = (index, &target.operand) {
                    let operand = writer.projected_entity(operand)?;
                    writer.unlink(Head::reference(), &owner, &member)?;
                    writer.unlink(Operand::reference(), &member, &operand)?;
                }
                writer.delete_entity(Member::reference(), &member)?;
                writer.retire_member::<Deleted>(key, &member)?;
            }
            for (key, note) in input.notes.iter().zip(&target.notes) {
                let note = writer.projected_entity(note)?;
                writer.unlink(NotedBy::reference(), &note, &owner)?;
                writer.delete_entity(Note::reference(), &note)?;
                writer.retire_member::<DeletedNotes>(key, &note)?;
            }
            if let (Some(row), Some(member)) = (&input.revalue, &target.revalued) {
                let member = writer.projected_entity(member)?;
                writer.write_field(&member, Value::at("MemberValue"), row.value)?;
                writer.preserve_member::<Kept>(&row.key, &member)?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
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
            let operand = match &input.operand {
                Some(key) => {
                    let operand = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                    Some(reader.mutation_target(&operand)?)
                }
                None => None,
            };
            let revalued = match &input.revalue {
                Some(row) => {
                    let member =
                        reader.resolve_entity(MemberKey::at("MemberKey"), row.key.clone())?;
                    reader.field(&member, Value::at("MemberValue"))?;
                    Some(reader.mutation_target(&member)?)
                }
                None => None,
            };
            Ok(Restoration {
                owner: reader.mutation_target(&owner)?,
                operand,
                revalued,
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
        use world::Fault::*;
        let result = (|| {
            let owner = writer.projected_entity(&target.owner)?;
            // A fault corrupts only the first returned member.
            for (index, row) in input.members.iter().enumerate() {
                let first = index == 0;
                let member = writer.create_entity_in_context(
                    &owner,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new(format!("returned-member.{}", row.key))
                        .unwrap(),
                )?;
                let key = if first && self.0 == ChangedRestoredKey {
                    "changed".into()
                } else {
                    row.key.clone()
                };
                writer.initialize_field(&member, MemberKey::at("MemberKey"), key)?;
                writer.initialize_field(
                    &member,
                    Value::at("MemberValue"),
                    row.value + u64::from(first && self.0 == ChangedRestoredValue),
                )?;
                let link = format!("returned-owner.{}", row.key);
                writer.link(OwnedBy::reference(), link, &member, &owner)?;
                if let (true, Some(operand)) = (first, &target.operand) {
                    let operand = writer.projected_entity(operand)?;
                    writer.link(Head::reference(), "returned-head", &owner, &member)?;
                    writer.link(Operand::reference(), "returned-operand", &member, &operand)?;
                }
                writer.create_member::<Restored>(&row.key, &member)?;
            }
            for row in &input.notes {
                let note = writer.create_entity_in_context(
                    &owner,
                    Note::reference(),
                    WorthQueryApplicationEntityKey::new(format!("returned-note.{}", row.key))
                        .unwrap(),
                )?;
                writer.initialize_field(&note, NoteKey::at("NoteKey"), row.key.clone())?;
                writer.initialize_field(&note, NoteValue::at("NoteValue"), row.value)?;
                let link = format!("returned-noted.{}", row.key);
                writer.link(NotedBy::reference(), link, &note, &owner)?;
                writer.create_member::<RestoredNotes>(&row.key, &note)?;
            }
            if let (Some(row), Some(member)) = (&input.revalue, &target.revalued) {
                let member = writer.projected_entity(member)?;
                writer.write_field(&member, Value::at("MemberValue"), row.value)?;
            }
            if matches!(self.0, RepeatedRestoredMember | ExtraRestoredMember) {
                let extra = writer.create_entity_in_context(
                    &owner,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new("additional-member").unwrap(),
                )?;
                // A repeat copies the first returned member; an extra is a member
                // the source never retired.
                let key = match input.members.first() {
                    Some(row) if self.0 == RepeatedRestoredMember => row.key.clone(),
                    _ => input.operand.clone().unwrap_or_else(|| "stray".into()),
                };
                let value = input.members.first().map_or(0, |row| row.value);
                writer.initialize_field(&extra, MemberKey::at("MemberKey"), key)?;
                writer.initialize_field(&extra, Value::at("MemberValue"), value)?;
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
