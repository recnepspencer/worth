//! A separate authored publication changes only the second region's outgoing adjacency.
use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Extend {
    pub id: u64,
}
worth_query_structured_value_binding!(pub Input for Extend { identity: "ownership.split-walk.extend.v1" });
worth_query_operation!(pub ExtendOp for S: Scenario, input Input);
impl<const CONTENT: bool> OperationReads<ExtendOp> for RawId<CONTENT> {}
impl OperationReads<ExtendOp> for ForkNumber {}
impl OperationReads<ExtendOp> for Branch {}
impl OperationCreates<ExtendOp> for Fork {}
impl OperationWrites<ExtendOp> for ForkNumber {}
impl OperationLinks<ExtendOp> for Branch {}
pub struct Outputs;
pub struct Created<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Created<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Fork;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-extension";
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<Created<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
type Targets<S> = (
    WorthQueryInvariantMutationTarget<S, Node<S>>,
    WorthQueryInvariantMutationTarget<S, Fork>,
);
worth_query_mutation_binding!(
    pub ExtendBinding for Extend, schema generic S: Scenario,
    identity "ownership.split-walk.command.v1", input Input, operation ExtendOp,
    result query::UnitBinding, idempotency String, identity "ownership.split-walk.key.v1",
    decision Targets<S>, denial query::UnitBinding, handler identity "ownership.split-walk.handler.v1",
    outputs Outputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>, field schema::id::<S>(), value_field id,
);
pub struct Handler;
impl<S: Scenario> OperationHandler<S, ExtendBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Extend,
        reader: &mut DecisionReader<'_, '_, '_, S, ExtendBinding<S>>,
    ) -> HandlerResult<Targets<S>, ()> {
        let result = (|| {
            let source = reader.resolve_entity(schema::id::<S>(), input.id)?;
            let second = reader.resolve_entity(ForkNumber::at("ForkNumber"), 2)?;
            reader
                .relations_from(Branch::reference(), &second)
                .map_err(HandlerExecutionDenial::new)?;
            Ok((
                reader.mutation_target(&source)?,
                reader.mutation_target(&second)?,
            ))
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Extend,
        (source, second): Targets<S>,
        writer: &mut CandidateWriter<'_, S, ExtendBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            let source = writer.projected_entity(&source)?;
            let second = writer.projected_entity(&second)?;
            let added = writer.create_entity_in_context(
                &source,
                Fork::reference(),
                WorthQueryApplicationEntityKey::new("external-region-extension").unwrap(),
            )?;
            writer.initialize_field(&added, ForkNumber::at("ForkNumber"), 3)?;
            writer.link(Branch::reference(), "extend-second-region", &second, &added)?;
            writer.create_output::<Created<S>>(&added)
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
    let op = ExtendOp::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_read_field(op, schema::id::<S>())
        .operation_read_field(op, ForkNumber::at("ForkNumber"))
        .operation_read_relation(op, Branch::reference())
        .operation_create(op, Fork::reference())
        .operation_write(op, ForkNumber::at("ForkNumber"))
        .operation_link(op, Branch::reference())
        .application_mutation_binding::<ExtendBinding<S>>()
}
