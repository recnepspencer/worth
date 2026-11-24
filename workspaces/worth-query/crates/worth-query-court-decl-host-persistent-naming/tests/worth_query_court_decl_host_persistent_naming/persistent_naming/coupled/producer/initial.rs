//! Initial: the producer's first run for a model creates a primary and a
//! secondary member, links a seam between them, and names all three through
//! one naming scope over the model's occurrence: the secondary a piece cut
//! by the primary, the seam told apart by the primary, both by the primary's
//! same-commit name.
use super::*;
use worth_query_host::facade::application_naming::NameRef;

worth_query_operation!(pub PiecesInitial for Schema, input InputBinding; access {
    read_entities: [Model, Member];
    read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    read_relations: [ModelMember, Seam];
    writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    creates: [Member]; links: [ModelMember, Seam];
});
pub type Operation = PiecesInitial;
pub struct Outputs;
pub struct Primary;
pub struct Secondary;
impl WorthQueryApplicationOutputRole for Primary {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "primary";
}
impl WorthQueryApplicationOutputRole for Secondary {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "secondary";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Primary as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Secondary as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}
pub struct Decision {
    source: WorthQueryInvariantMutationTarget<Schema, Model>,
    /// Committed names the decision read on the candidate (`reads`).
    cut: Option<super::reads::Cut>,
}
producer_binding!("persistent-naming.pieces.initial", Initial, Primary);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let source = reader.resolve_entity(model_key(), input.key.clone())?;
            reader.field(&source, ModelValue::at("ModelValue"))?;
            let prior = reader.resolve_optional_entity(
                MemberKey::at("MemberKey"),
                super::primary_key(&input.key),
            )?;
            if prior.is_some() {
                return Ok(Err(edit::Denial::ModelMismatch));
            }
            let cut = super::reads::decide(reader, input.value)?;
            Ok(Ok(Decision {
                source: reader.mutation_target(&source)?,
                cut,
            }))
        })();
        match result {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(denial)) => HandlerResult::DomainDenied(denial),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let source = writer.projected_entity(&decision.source)?;
            let mut created = Vec::new();
            for (key, value) in [
                (super::primary_key(&input.key), input.value),
                (super::secondary_key(&input.key), input.value + 1),
            ] {
                let member = writer.create_entity_in_context(
                    &source,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
                )?;
                writer.initialize_field(&member, MemberKey::at("MemberKey"), key.clone())?;
                writer.initialize_field(&member, MemberValue::at("MemberValue"), value)?;
                writer.link(
                    ModelMember::reference(),
                    format!("owns:{key}"),
                    &source,
                    &member,
                )?;
                created.push(member);
            }
            let (primary, secondary) = (&created[0], &created[1]);
            let occurrence = OccurrencePath::root(&super::occurrence(&input.key)).unwrap();
            let mut names = writer.naming_scope(occurrence);
            let birth = names.birth(MemberRole::Authored);
            names.name_entity(primary, birth)?;
            if super::naming(&input.key) == super::Naming::Complete {
                // Cut by the member and along the seam the decision read, or
                // else by the primary born beside it.
                let cut = match &decision.cut {
                    Some(read) => outputs::SidedCut::By {
                        cutter: read.member.into(),
                        side: outputs::CutSide::Positive,
                        along: Some(read.seam.into()),
                    },
                    None => outputs::SidedCut::By {
                        cutter: names.same_commit_entity_name(primary)?.into(),
                        side: outputs::CutSide::Positive,
                        along: None::<NameRef<Seam>>,
                    },
                };
                let birth = names.birth(MemberRole::Piece).with_discriminator(&cut);
                names.name_entity(secondary, birth)?;
            }
            let member = names.same_commit_entity_name(primary)?;
            let birth = names
                .birth(SeamRole::Seam)
                .with_discriminator(&outputs::SeamOf { index: 0, member });
            names.link_named(
                Seam::reference(),
                super::seam_key(&input.key),
                primary,
                secondary,
                birth,
            )?;
            drop(names);
            writer.create_output::<Primary>(primary)?;
            writer.create_output::<Secondary>(secondary)?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
