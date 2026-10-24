//! Owner logic only: public correction courts separately prove actual source material.
use super::super::fixture::*;
use crate::domain_computation::primary_graph::product_operation::correction_source::material::{
    finish_derivation,
    source_outputs::{
        read_checked, validate_witness, WorthQueryCorrectionOutputProjectionDenial as D,
    },
};
use crate::domain_computation::primary_graph::WorthQueryCorrectionDerivationDenial;
use std::{any::TypeId, cell::RefCell};
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::{
    application_aftermath::*, application_operation::*, application_schema::*,
};

type S = IdentityExecutionSchema;
type Base = ProgramRequiredMutationBinding;
struct CreatedSource;
struct Outputs;
struct Created;
impl ApplicationMutationOutputContract<S> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Created::DESCRIPTOR];
}
impl WorthQueryApplicationOutputRole for Created {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-account";
}
// Only the output contract differs. This source is used to construct typed
// witnesses; it is not installed or presented as a performed source occurrence.
impl ApplicationMutationBinding<S> for CreatedSource {
    type Input = <Base as ApplicationMutationBinding<S>>::Input;
    type InputBinding = <Base as ApplicationMutationBinding<S>>::InputBinding;
    type Operation = <Base as ApplicationMutationBinding<S>>::Operation;
    type Result = <Base as ApplicationMutationBinding<S>>::Result;
    type ResultBinding = <Base as ApplicationMutationBinding<S>>::ResultBinding;
    type IdempotencyKey = <Base as ApplicationMutationBinding<S>>::IdempotencyKey;
    type Decision = <Base as ApplicationMutationBinding<S>>::Decision;
    type Denial = <Base as ApplicationMutationBinding<S>>::Denial;
    type DenialBinding = <Base as ApplicationMutationBinding<S>>::DenialBinding;
    type Output = Outputs;
    type ScopeBinding = <Base as ApplicationMutationBinding<S>>::ScopeBinding;
    type PrincipalBinding = <Base as ApplicationMutationBinding<S>>::PrincipalBinding;
    type Mapping = <Base as ApplicationMutationBinding<S>>::Mapping;
    type Principal = <Base as ApplicationMutationBinding<S>>::Principal;
    type PrincipalIdentity = <Base as ApplicationMutationBinding<S>>::PrincipalIdentity;
    type PrincipalIdentityBinding =
        <Base as ApplicationMutationBinding<S>>::PrincipalIdentityBinding;
    type SourceExpectation = <Base as ApplicationMutationBinding<S>>::SourceExpectation;
    const IDENTITY: &'static str = "correction-output.preflight-source.v1";
    const HANDLER_IDENTITY: &'static str = Base::HANDLER_IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = Base::IDEMPOTENCY_IDENTITY;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        Base::workflow_execution_requirements()
    }
    fn scope_field() -> ApplicationFieldRef<
        S,
        Account,
        AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Base::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    > {
        Base::principal_binding()
    }
}

struct OtherNativeCodec;
impl ApplicationScalarValueBinding for OtherNativeCodec {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = StringApplicationValueBinding::IDENTITY_NAME;
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &String) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(value)
    }
    fn encode(value: &String) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for OtherNativeCodec {
    fn decode(value: &AspectValue) -> Result<String, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value)
    }
}
struct SameNamedField;
impl ApplicationFieldMarkerIdentity<S, Account, AccountPolicy> for SameNamedField {
    const IDENTIFIER: &'static str =
        <AccountStatus as ApplicationFieldMarkerIdentity<S, Account, AccountPolicy>>::IDENTIFIER;
}
impl DeclaredApplicationFieldValue for SameNamedField {
    type Value = String;
    type Binding = OtherNativeCodec;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}
fn field() -> ApplicationCorrectionOutputField<S> {
    ApplicationCorrectionOutput::<CreatedSource, Created>::field(AccountStatus::reference())
}
fn other_codec() -> ApplicationCorrectionOutputField<S> {
    ApplicationCorrectionOutput::<CreatedSource, Created>::field(ApplicationFieldRef::<
        S,
        Account,
        AccountPolicy,
        SameNamedField,
        String,
        ReadWrite,
        EqualityPredicate,
        NoApplicationUnit,
    >::from_schema_types())
}

#[test]
fn correction_output_preflight_checks_source_manifest_and_native_codec() {
    let world = installed_authorization_world(true);
    let catalog = world.application.installed_schema().value_bindings();
    let field = field();
    let manifest = [field.declaration().clone()];
    let source = TypeId::of::<CreatedSource>();
    validate_witness(source, &manifest, catalog, &field).unwrap();
    assert!(matches!(
        validate_witness(TypeId::of::<Base>(), &manifest, catalog, &field),
        Err(D::ForeignWitness)
    ));
    let undeclared =
        ApplicationCorrectionOutput::<CreatedSource, Created>::field(AccountLabel::reference());
    assert!(matches!(
        validate_witness(source, &manifest, catalog, &undeclared),
        Err(D::UndeclaredField)
    ));
    let other = other_codec();
    assert_eq!(
        other.declaration(),
        field.declaration(),
        "portable field meaning is identical"
    );
    assert_ne!(
        other.binding_recipe().binding_type(),
        field.binding_recipe().binding_type()
    );
    assert!(matches!(
        validate_witness(source, &manifest, catalog, &other),
        Err(D::FieldMeaningMismatch)
    ));
}

#[test]
fn correction_output_preflight_refusal_survives_a_successful_callback() {
    let world = installed_authorization_world(true);
    let catalog = world.application.installed_schema().value_bindings();
    let field = field();
    let manifest = [field.declaration().clone()];
    let failure = RefCell::new(None);
    let mut calls = 0;
    let mut callback = || {
        let ignored = read_checked(&failure, || {
            validate_witness(
                TypeId::of::<CreatedSource>(),
                &manifest,
                catalog,
                &other_codec(),
            )
        });
        assert!(matches!(
            ignored,
            Err(ApplicationCorrectionSourceFieldDenial::Refused)
        ));
        let refused_again = read_checked(&failure, || {
            calls += 1;
            Ok(())
        });
        assert!(refused_again.is_err());
        Ok::<_, ApplicationCorrectionDerivationDenial>("plausible corrective intent")
    };
    let result = callback();
    assert_eq!(calls, 0, "a failed reader performs no further field reads");
    assert!(matches!(
        finish_derivation(result, failure.into_inner()),
        Err(WorthQueryCorrectionDerivationDenial::SourceOutput(
            D::FieldMeaningMismatch
        ))
    ));
    assert_eq!(
        finish_derivation(Ok("lawful twin"), None).unwrap(),
        "lawful twin"
    );
}
