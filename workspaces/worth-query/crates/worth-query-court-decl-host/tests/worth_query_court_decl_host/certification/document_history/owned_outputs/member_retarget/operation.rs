//! Preserve the owner and change only the actual arriving/leaving input members.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub owner: String,
    pub value: u64,
    pub retire: Vec<String>,
    pub create: Vec<Row>,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.member-retarget-input" });
worth_query_operation!(pub Revise for Schema, input Input; access {
    read_entities: [Model, InputMember]; read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), InputKey::at("InputKey"), InputValue::at("InputValue")];
    read_relations: [InputOwner]; writes: [ModelValue::at("ModelValue"), InputKey::at("InputKey"), InputValue::at("InputValue")]; creates: [InputMember]; deletes: [InputMember]; links: [InputOwner]; unlinks: [InputOwner];
});
pub struct Outputs;
pub struct Owner;
impl WorthQueryApplicationOutputRole for Owner {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Model;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "input-owner";
}
pub struct Leaving;
pub struct Arriving;
macro_rules! family {
    ($name:ident, $posture:ident, $prefix:literal) => {
        impl WorthQueryApplicationOutputRoleFamily for $name {
            type Schema = Schema;
            type Contract = Outputs;
            type Entity = InputMember;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::$posture;
            const MINIMUM: usize = 0;
        }
    };
}
family!(Leaving, RETIRE, "leaving-input.");
family!(Arriving, CREATE, "arriving-input.");
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Owner::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[Leaving::DESCRIPTOR, Arriving::DESCRIPTOR];
}
pub struct Decision {
    owner: WorthQueryInvariantMutationTarget<Schema, Model>,
    leaving: Vec<WorthQueryInvariantMutationTarget<Schema, InputMember>>,
}
fn scope(input: &Intent) -> String {
    input.owner.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.member-retarget", input Input, operation Revise, result edit::ResultBinding,
    idempotency String, identity "owned-outputs.member-retarget", decision Decision, denial edit::DenialBinding,
    handler identity "owned-outputs.member-retarget", program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let owner = reader.resolve_entity(ModelKey::at("ModelKey"), input.owner.clone())?;
            reader.field(&owner, ModelValue::at("ModelValue"))?;
            for row in &input.create {
                // The native unique owner needs the indexed selection, including absence.
                reader.resolve_optional_entity(InputKey::at("InputKey"), row.key.clone())?;
            }
            let mut leaving = Vec::new();
            for key in &input.retire {
                let member = reader.resolve_entity(InputKey::at("InputKey"), key.clone())?;
                reader.field(&member, InputKey::at("InputKey"))?;
                reader.field(&member, InputValue::at("InputValue"))?;
                let links = reader
                    .relations_from(InputOwner::reference(), &member)
                    .map_err(HandlerExecutionDenial::new)?;
                assert_eq!(links.len(), 1);
                assert_eq!(
                    reader.field(links[0].to(), ModelKey::at("ModelKey"))?,
                    Some(input.owner.clone())
                );
                leaving.push(reader.mutation_target(&member)?);
            }
            Ok(Decision {
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
        input: &Intent,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let owner = writer.projected_entity(&decision.owner)?;
            writer.write_field(&owner, ModelValue::at("ModelValue"), input.value)?;
            for (key, member) in input.retire.iter().zip(&decision.leaving) {
                let member = writer.projected_entity(member)?;
                writer.unlink(InputOwner::reference(), &member, &owner)?;
                writer.delete_entity(InputMember::reference(), &member)?;
                writer.retire_member::<Leaving>(key, &member)?;
            }
            for row in &input.create {
                let member = writer.create_entity_in_context(
                    &owner,
                    InputMember::reference(),
                    WorthQueryApplicationEntityKey::new(format!("input:{}", row.key)).unwrap(),
                )?;
                writer.initialize_field(&member, InputKey::at("InputKey"), row.key.clone())?;
                writer.initialize_field(&member, InputValue::at("InputValue"), row.value)?;
                writer.link(
                    InputOwner::reference(),
                    format!("input-owner:{}", row.key),
                    &member,
                    &owner,
                )?;
                writer.create_member::<Arriving>(&row.key, &member)?;
            }
            writer.preserve_output::<Owner>(&owner)?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
