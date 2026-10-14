//! Adoption uses a distinct action and grant on the existing scope grant owner.
use super::capability::*;
use super::*;
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_program::*;
pub(in super::super) struct ProgramAdoptionCapability<Schema>(PhantomData<fn() -> Schema>);
impl<Schema> worth_query_decl::facade::portable_identity::WorthQueryPortableType
    for ProgramAdoptionCapability<Schema>
{
    const PORTABLE_TYPE_NAME: &'static str =
        "worth.query.certification.oracle-adoption-capability.v1";
}
impl<Schema: TopologySchemaBinding> ApplicationCapabilityMarkerIdentity
    for ProgramAdoptionCapability<Schema>
{
    type Schema = Schema;
    const IDENTIFIER: &'static str = "ProgramAdoptionCapability";
}
impl<Schema: TopologySchemaBinding> ProgramAdoptionCapability<Schema> {
    fn reference() -> ApplicationCapabilityRef<Schema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
#[derive(serde::Serialize)]
pub(in super::super) struct ProgramAdoptionInput {
    source: String,
    target: String,
}
worth_query_structured_value_binding!(pub(in super::super) ProgramAdoptionInputBinding for ProgramAdoptionInput { identity: "worth.query.certification.oracle-adoption-input.v1" });
worth_query_operation!(pub(in super::super) ProgramAdoptionOperation for Schema: TopologySchemaBinding, input ProgramAdoptionInputBinding);
worth_query_operation_reads!(ProgramAdoptionOperation => [BodyKey]);
pub(in super::super) struct ProgramAdoptionBinding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationProgramAdoptionBinding<Schema>
    for ProgramAdoptionBinding<Schema>
{
    const IDENTITY: &'static str = "worth.query.certification.oracle-adoption-binding.v1";
    type Capability = ProgramAdoptionCapability<Schema>;
    type Input = ProgramAdoptionInput;
    type InputBinding = ProgramAdoptionInputBinding;
    type Operation = ProgramAdoptionOperation;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> ProgramAdoptionInput {
        ProgramAdoptionInput {
            source: context.source.to_string(),
            target: context.target.to_string(),
        }
    }
}
impl<Schema: TopologySchemaBinding>
    ApplicationCapabilityRequest<Schema, ProgramAdoptionCapability<Schema>>
    for ProgramAdoptionInput
{
    type Scope = Body;
    type Context = CorrectionContext<Schema>;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Body, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let scalar =
            || ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(2).unwrap();
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                BodyKey::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    super::super::oracle::installation::SCOPE.to_owned(),
                )
                .unwrap(),
            ),
            scalar(),
            scalar(),
            ApplicationCapabilityRequestContext::new(CorrectionContext::<Schema>::reference()),
        ))
    }
}
pub(in super::super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .operation(
            ProgramAdoptionOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ProgramAdoptionOperation::reference(), 512)
        .operation_read_field(ProgramAdoptionOperation::reference(), BodyKey::reference())
        .capability(super::capability::contract::grant_contract(
            ProgramAdoptionCapability::<Schema>::reference(),
            ProgramAdoptionOperation::reference(),
            2,
            2,
        ))
}
