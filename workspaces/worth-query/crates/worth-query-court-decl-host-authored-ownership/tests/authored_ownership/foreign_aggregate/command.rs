//! An independent authored publication creates the foreign orbit member.
use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Birth {
    pub id: u64,
}
worth_query_structured_value_binding!(pub Input for Birth { identity:"ownership.foreign-aggregate.birth.v1" });
worth_query_operation!(pub BirthOp for S:Scenario,input Input);
impl OperationReads<BirthOp> for Id {}
impl OperationCreates<BirthOp> for Occurrence {}
impl OperationWrites<BirthOp> for Number {}
impl OperationLinks<BirthOp> for Next {}
pub struct Outputs;
pub struct Created<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Created<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Occurrence;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "foreign-authored-member";
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<Created<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
worth_query_mutation_binding!(
 pub BirthBinding for Birth,schema generic S:Scenario,
 identity "ownership.foreign-aggregate.birth-command.v1",input Input,operation BirthOp,
 result query::UnitBinding,idempotency String,identity "ownership.foreign-aggregate.key.v1",
 decision WorthQueryInvariantMutationTarget<S,Node>,denial query::UnitBinding,handler identity "ownership.foreign-aggregate.handler.v1",
 outputs Outputs,source NoApplicationMutationSource,
 principal binding actor(),binding_type Actor,mapping Mapping,principal_entity Node,
 principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Node,Facts,Id,u64,ReadOnly,NoApplicationUnit,field Id::reference(),value_field id,
);
pub struct Handler;
impl<S: Scenario> OperationHandler<S, BirthBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Birth,
        reader: &mut DecisionReader<'_, '_, '_, S, BirthBinding<S>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<S, Node>, ()> {
        let result = reader
            .resolve_entity(Id::reference(), input.id)
            .and_then(|source| reader.mutation_target(&source));
        match result {
            Ok(source) => HandlerResult::Completed(source),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Birth,
        source: WorthQueryInvariantMutationTarget<S, Node>,
        writer: &mut CandidateWriter<'_, S, BirthBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            let source = writer.projected_entity(&source)?;
            let foreign = writer.create_entity_in_context(
                &source,
                Occurrence::reference(),
                WorthQueryApplicationEntityKey::new("foreign-authored-orbit-member").unwrap(),
            )?;
            writer.initialize_field(&foreign, Number::reference(), 1)?;
            writer.link(
                Next::reference(),
                "foreign-authored-self-loop",
                &foreign,
                &foreign,
            )?;
            writer.create_output::<Created<S>>(&foreign)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = BirthOp::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_read_field(op, Id::reference())
        .operation_create(op, Occurrence::reference())
        .operation_write(op, Number::reference())
        .operation_link(op, Next::reference())
        .application_mutation_binding::<BirthBinding<S>>()
}
