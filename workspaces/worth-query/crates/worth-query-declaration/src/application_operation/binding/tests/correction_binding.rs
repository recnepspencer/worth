//! One contribution clause owns capture metadata and native typed dispatch.
use super::*;
use crate::application_aftermath::*;
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_operation::*;
use crate::application_schema::*;
mod disclosure;

struct Inverse;
crate::worth_query_operation_reads!(RenameOperation => [AccountIdField]);
impl ApplicationDeclaredOperation<MutationSchema> for RenameOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<MutationSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<MutationSchema> {
        schema.operation_read_field(Self::reference(), AccountIdField::reference())
    }
}
impl ApplicationCorrectionBinding<MutationSchema> for Inverse {
    const IDENTITY: &'static str = "test.rename-correction.v1";
    type Source = RenameBinding;
    type InverseIntent = RenameInput;
    type Disclosure = disclosure::HistoryBinding;

    fn disclosure_parameters(
        _: &RenameInput,
    ) -> Result<
        crate::application_query::ApplicationQueryParameterSet<disclosure::HistoryQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(crate::application_query::ApplicationQueryParameterSet::new())
    }

    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        MutationSchema,
        RenameOperation,
        RenameInput,
        true,
        false,
    > {
        RenameOperation::reference()
            .definition()
            .no_external_effect()
    }

    fn preimage_demand() -> DeclaredPreImageDemand<MutationSchema> {
        DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(
                AccountIdField::reference(),
            )],
            64,
        )
        .unwrap()
    }

    fn derive_inverse(
        _: &RenameInput,
        _: &impl ApplicationCorrectionPreImage<MutationSchema>,
    ) -> Result<RenameInput, ApplicationCorrectionDerivationDenial> {
        // This owner test exercises registration, not a domain inverse implementation.
        Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning)
    }

    fn inverse_key(key: &str) -> u64 {
        key.parse().unwrap()
    }

    fn visit<Visitor: ApplicationCorrectionVisitor<MutationSchema>>(
        dispatch: ApplicationCorrectionDispatch<MutationSchema, Self, Visitor>,
    ) -> Visitor::Output {
        dispatch.conventional()
    }
}

crate::worth_query_application_contribution! {
    contribution CorrectionContribution in MutationSchema {
        identity: "test.correction-contribution.v1",
        corrections: [Inverse],
        members: |schema| {
            schema
                .entity(ExternalMapping::reference()).entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Account::reference(), AccountIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Account::reference(), AccountIdField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Account::reference())
                .principal_binding(PrincipalBinding::reference())
        }
    }
}

#[test]
fn contribution_correction_clause_registers_exact_binding_capture_and_dispatch() {
    let _reference = CorrectionContribution::reference();
    let declaration =
        CorrectionContribution::register_members(ApplicationSchemaDeclarationBuilder::<
            MutationSchema,
        >::for_schema())
        .build()
        .unwrap();
    let [binding] = declaration.member_provenance().mutation_bindings() else {
        panic!("a self-inverse association registers its native binding once")
    };
    assert_eq!(binding.binding_type(), TypeId::of::<RenameBinding>());
    let contract = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::OperationAftermath { contract, .. } => Some(contract),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        contract.inverse_binding(),
        Some(&DeclaredInverseBinding::of::<MutationSchema, Inverse>())
    );
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = contract.mechanism() else {
        panic!("the same clause installs capture demand")
    };
    assert_eq!(
        inverse.preimage_demand().loci()[0].field(),
        "AccountIdField"
    );
    assert!(declaration
        .erased()
        .members()
        .iter()
        .any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead {
                operation, target: ApplicationOperationDecisionReadTarget::Field { field, .. },
            } if operation == "RenameOperation" && field == "AccountIdField"
        )));
    assert_eq!(
        CorrectionContribution::visit_correction(TypeId::of::<RenameBinding>(), BindingVisitor)
            .unwrap(),
        Some(TypeId::of::<RenameBinding>())
    );
    assert!(CorrectionContribution::visit_correction(TypeId::of::<()>(), BindingVisitor).is_err());
}

#[derive(Debug)]
struct BindingVisitor;

#[test]
fn correction_identities_share_the_input_used_for_execution_without_a_payload_copy() {
    let input =
        ApplicationEncodedInput::<RenameInputBinding>::encode(RenameInput { account_id: 41 })
            .unwrap();
    let original_pointer = input.input() as *const RenameInput;
    let original_identity = *input.identity();
    let key = 7;
    let encoded_key =
        ApplicationEncodedMutationKey::<MutationSchema, RenameBinding>::encode(key).unwrap();
    let key_identity = *encoded_key.identity();
    let owned = encoded_key.with_input(&input);
    assert_eq!(owned.borrow().key_identity(), &key_identity);
    let identities =
        ApplicationMutationIdentities::<MutationSchema, RenameBinding>::with_encoded_input(
            &key, &input,
        )
        .unwrap();
    drop(input);
    assert_eq!(
        identities.mutation_input() as *const RenameInput,
        original_pointer
    );
    let retained = identities.encoded_input();
    drop(identities);
    assert_eq!(
        owned.borrow().mutation_input() as *const RenameInput,
        original_pointer
    );
    assert_eq!(owned.borrow().input_identity(), &original_identity);
    drop(owned);
    assert_eq!(retained.input() as *const RenameInput, original_pointer);
    assert_eq!(retained.identity(), &original_identity);
    assert_eq!(retained.input().account_id, 41);
}

impl ApplicationCorrectionVisitor<MutationSchema> for BindingVisitor {
    type Output = Option<TypeId>;
    fn unavailable(self) -> Self::Output {
        None
    }

    fn conventional<Correction: ApplicationCorrectionBinding<MutationSchema>>(self) -> Self::Output
    where
        ApplicationInverseBinding<MutationSchema, Correction>: ApplicationMutationAdmissionBinding<
            MutationSchema,
            Admission = ApplicationConventionalAdmission,
        >,
        <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<
            MutationSchema,
        >>::Input: Clone + Send + Sync,
        <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<
            MutationSchema,
        >>::ScopeBinding: ApplicationMutationScopeResolution<
            MutationSchema,
            <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<
                MutationSchema,
            >>::PrincipalIdentity,
        >,
    {
        Some(TypeId::of::<Correction::Source>())
    }

    fn capability<Correction, Capability>(self) -> Self::Output
    where
        Correction: ApplicationCorrectionBinding<MutationSchema>,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = MutationSchema> + 'static,
        ApplicationInverseBinding<MutationSchema, Correction>: ApplicationCapabilityMutationBinding<MutationSchema, Capability = Capability>,
        <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<MutationSchema>>::Input: Clone + Send + Sync + ApplicationCapabilityRequest<
            MutationSchema, Capability,
            Scope = <<ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<MutationSchema>>::ScopeBinding as ApplicationMutationScopeBinding<MutationSchema>>::Scope,
        >,
        <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<MutationSchema>>::ScopeBinding: ApplicationMutationScopeResolution<
            MutationSchema, <ApplicationInverseBinding<MutationSchema, Correction> as ApplicationMutationBinding<MutationSchema>>::PrincipalIdentity,
        >,
    {
        Some(TypeId::of::<Correction::Source>())
    }
}
