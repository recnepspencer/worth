//! Preserve: a later edit of the model re-derives the producer's outputs in
//! place. It writes their values and creates nothing, so it names nothing:
//! both members keep the names the Initial minted.
use super::*;

worth_query_operation!(pub PiecesPreserve for Schema, input InputBinding; access {
    read_entities: [Model, Member];
    read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    read_relations: [ModelMember, Seam];
    writes: [MemberValue::at("MemberValue")];
});
pub type Operation = PiecesPreserve;
pub struct Outputs;
pub struct Primary;
pub struct Secondary;
impl WorthQueryApplicationOutputRole for Primary {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "primary";
}
impl WorthQueryApplicationOutputRole for Secondary {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryPreserveOutput;
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
    primary: WorthQueryInvariantMutationTarget<Schema, Member>,
    secondary: WorthQueryInvariantMutationTarget<Schema, Member>,
}
producer_binding!("persistent-naming.pieces.preserve", Preserve, Primary);

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
            let primary = reader.prior_output_preserved_or_created::<
                Binding,
                Primary,
                super::initial::Binding,
                super::initial::Primary,
            >()?;
            let secondary = reader.prior_output_preserved_or_created::<
                Binding,
                Secondary,
                super::initial::Binding,
                super::initial::Secondary,
            >()?;
            reader.field(&primary, MemberValue::at("MemberValue"))?;
            reader.field(&secondary, MemberValue::at("MemberValue"))?;
            Ok::<_, HandlerExecutionDenial>(Decision {
                primary: reader.mutation_target(&primary)?,
                secondary: reader.mutation_target(&secondary)?,
            })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
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
            let primary = writer.projected_entity(&decision.primary)?;
            let secondary = writer.projected_entity(&decision.secondary)?;
            writer.write_field(&primary, MemberValue::at("MemberValue"), input.value)?;
            writer.write_field(&secondary, MemberValue::at("MemberValue"), input.value + 1)?;
            writer.preserve_output::<Primary>(&primary)?;
            writer.preserve_output::<Secondary>(&secondary)?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
