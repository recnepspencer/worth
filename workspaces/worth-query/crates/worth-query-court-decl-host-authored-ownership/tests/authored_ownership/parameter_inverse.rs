//! A parameter edit crosses real owned producer succession history.
use super::{lifecycle::*, *};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Edit {
    pub id: u64,
    pub value: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Edit {identity:"ownership.parameter.input.v1"});
worth_query_operation!(pub EditOp for S:Scenario,input InputBinding);
impl<const CONTENT: bool> OperationReads<EditOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<EditOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<EditOp> for RawValue<CONTENT> {}
impl OperationReads<EditOp> for Peer {}
impl<const CONTENT: bool> OperationWrites<EditOp> for RawValue<CONTENT> {}
worth_query_mutation_binding!(
 pub Binding for Edit,schema generic S:Scenario,
 identity "ownership.parameter-edit.v1",input InputBinding,operation EditOp,
 result query::UnitBinding,idempotency String,identity "ownership.parameter.command.v1",
 decision Target<S>,denial query::UnitBinding,handler identity "ownership.parameter.handler.v1",
 outputs NoApplicationMutationOutputs,source NoApplicationMutationSource,
 principal binding actor(),binding_type Actor,mapping Mapping,principal_entity Node<S>,
 principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Id<S>, field schema::id::<S>(),value_field id,
);
pub struct Handler;
impl<S: Scenario> OperationHandler<S, Binding<S>> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, S, Binding<S>>,
    ) -> HandlerResult<Target<S>, ()> {
        let result = (|| {
            let target = reader.resolve_entity(schema::id::<S>(), input.id)?;
            reader.field(&target, schema::value::<S>())?;
            reader
                .relations_to(Peer::reference(), &target)
                .map_err(HandlerExecutionDenial::new)?;
            reader.mutation_target(&target)
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        target: Target<S>,
        writer: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            let target = writer.projected_entity(&target)?;
            writer.write_field(&target, schema::value::<S>(), input.value)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
impl<S: Scenario> ApplicationDeclaredOperation<S> for EditOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_read_entity(Self::reference(), schema::node::<S>())
            .operation_read_field(Self::reference(), schema::id::<S>())
            .operation_read_field(Self::reference(), schema::value::<S>())
            .operation_read_relation(Self::reference(), Peer::reference())
            .operation_write(Self::reference(), schema::value::<S>())
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if !matches!(S::MODE, 48 | 68 | 69 | 70) {
        return schema;
    }
    schema
        .declared_operation(
            EditOp::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding<S>>()
}
