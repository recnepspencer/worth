//! A plain authored parameter inverse crosses real owned producer succession history.
use super::{lifecycle::*, *};
use std::marker::PhantomData;
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
        .application_mutation_binding::<Binding<S>>()
        .correction::<Inverse<S>>()
}
pub struct Inverse<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationCorrectionBinding<S> for Inverse<S> {
    fn recover_source_input(bytes: &[u8], expected: worth_query_decl::facade::application_schema::CanonicalDigestId, bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds) -> Result<worth_query_decl::facade::application_operation::ApplicationEncodedInput<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding>, worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial>{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds))
    }

    fn recover_inverse_input(bytes: &[u8], expected: worth_query_decl::facade::application_schema::CanonicalDigestId, bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds) -> Result<worth_query_decl::facade::application_operation::ApplicationEncodedInput<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding>, worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial>{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds))
    }

    const IDENTITY: &'static str = "ownership.parameter-inverse.v1";
    type Source = Binding<S>;
    type InverseIntent = Edit;
    type Disclosure = query::Binding<S>;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<S, EditOp, Edit, true, false> {
        EditOp::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Edit,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(schema::value::<S>())],
            9,
        ))
    }
    fn derive_correction(
        original: &Edit,
        material: ApplicationCorrectionSourceMaterial<'_, S>,
    ) -> Result<Edit, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let value = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(schema::value::<S>()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(Edit {
            id: original.id,
            value,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("inverse:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reapply:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<S>>(
        dispatch: ApplicationCorrectionDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn visit_reapplication<V: ApplicationReapplicationVisitor<S>>(
        dispatch: ApplicationReapplicationDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
#[test]
fn worker_only_recorded_parameter_inverse_rederives_through_the_owned_predecessor() {
    use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
 use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    type S = Model<48>;
    use worth_foundational::facade::{
        DeterminismContract, ExecutionPosture, ExecutionRequestPolicy,
    };
    use worth_query_host::facade::{
        admission::authenticated_principal::WorthQueryAdmissionClock, runtime,
    };
    let clock = WorthQueryAdmissionClock::system();
    // for_workers carries no accounting authority. A budget with unbounded
    // limits still carries accounting and cannot reproduce the C6 request.
    let resources =
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
            clock,
        )
        .with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ));
    assert!(resources.execution_policy().accounting().is_none());
    let limits = WorthQueryApplicationLimits::new(
        resources,
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap(),
        ),
        SignalConditionalEvaluationBudget::development(),
    );
    let host = super::correction_courts::try_install_definition_with_limits::<
        S,
        super::coupled::multi::TwoProgram,
    >(limits)
    .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    super::coupled::multi::create(&host, &request, 10, vec![20]);
    let edit = |value, key: &str| {
        let outcome = request
            .advanced()
            .mutate(Edit { id: 10, value })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        receipt
    };
    edit(3, "initial-parameter");
    super::coupled::multi::settle(&host, &request, 10);
    let original = edit(7, "parameter-to-seven");
    super::coupled::producer::handler::reset_contacts();
    super::coupled::multi::settle(&host, &request, 10);
    assert_eq!(super::coupled::producer::handler::contacts(), 1);
    let inverse = super::correction_courts::reverse(&request, &original, "parameter-back-to-three");
    assert_eq!(
        inverse
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        original.commit_reference()
    );
    assert_eq!(
        request
            .query(query::Input { id: 10 })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3
    );
    super::coupled::multi::settle(&host, &request, 10);
    assert_eq!(super::coupled::producer::handler::contacts(), 2);
    assert!(request.query(query::Input { id: 101 }).execute().is_ok());
}

mod adjacency_gap;
