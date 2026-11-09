//! Exact authored Retire role, admitted by the installed inverse declaration.
use super::super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent;
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.authored-member-retire.v1" });
worth_query_operation!(pub RetireMember for Schema, input Input; access {
    read_entities: [Model, Member, ChildOutput]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; read_relations: [ModelMember, ChildOf]; deletes: [Member, ChildOutput]; unlinks: [ModelMember, ChildOf];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.authored-member-retire.v1", input Input, operation RetireMember, result member_edit::ResultBinding,
    idempotency String, identity "owned-outputs.authored-member-retire.v1", decision Targets, denial member_edit::DenialBinding,
    handler identity "owned-outputs.authored-member-retire.v1", program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Outputs;
pub struct Retired;
impl WorthQueryApplicationOutputRole for Retired {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-member-retired";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Retired as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <DerivedRetired as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
pub struct DerivedRetired;
impl WorthQueryApplicationOutputRole for DerivedRetired {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = ChildOutput;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "derived-member-retired";
}
pub struct Targets {
    member: WorthQueryInvariantMutationTarget<Schema, Member>,
    owners: Vec<WorthQueryInvariantMutationTarget<Schema, Model>>,
    children: Vec<WorthQueryInvariantMutationTarget<Schema, ChildOutput>>,
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Targets, member_edit::Denial> {
        let result = (|| {
            let member = reader.resolve_entity(MemberKey::at("MemberKey"), String::from("m"))?;
            reader.entity(&member, Member::reference())?;
            reader.field(&member, MemberKey::at("MemberKey"))?;
            reader.field(&member, MemberValue::at("MemberValue"))?;
            let mut owners = Vec::new();
            for edge in reader
                .relations_to(ModelMember::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?
            {
                reader.entity(edge.from(), Model::reference())?;
                owners.push(reader.mutation_target(edge.from())?);
            }
            let mut children = Vec::new();
            for edge in reader
                .relations_from(ChildOf::reference(), &member)
                .map_err(HandlerExecutionDenial::new)?
            {
                reader.entity(edge.to(), ChildOutput::reference())?;
                children.push(reader.mutation_target(edge.to())?);
            }
            Ok(Targets {
                member: reader.mutation_target(&member)?,
                owners,
                children,
            })
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        targets: Targets,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<member_edit::Edited, member_edit::Denial> {
        let result = (|| {
            let member = writer.projected_entity(&targets.member)?;
            for owner in &targets.owners {
                let owner = writer.projected_entity(owner)?;
                writer.unlink(ModelMember::reference(), &owner, &member)?;
            }
            for child in &targets.children {
                let child = writer.projected_entity(child)?;
                writer.unlink(ChildOf::reference(), &member, &child)?;
                writer.delete_entity(ChildOutput::reference(), &child)?;
                writer.retire_output::<DerivedRetired>(&child)?;
            }
            writer.delete_entity(Member::reference(), &member)?;
            writer.retire_output::<Retired>(&member)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(member_edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
