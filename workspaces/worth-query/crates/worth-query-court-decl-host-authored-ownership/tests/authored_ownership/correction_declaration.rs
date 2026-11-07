//! Infallible correction expressions are refused by their schema composition.
use super::{lifecycle::*, *};
use worth_query_decl::facade::application_schema::fragment::*;

type S = Model<0>;
struct Correction<const DAMAGE: u8>;
struct Family<const DAMAGE: u8>;
impl<const DAMAGE: u8> ApplicationAuthoredCorrectionFamily<S> for Family<DAMAGE> {
    const IDENTITY: &'static str = "declaration.family.v1";
    type Entity = NativeNode<false>;
    fn logical_key() -> ApplicationAuthoredLogicalKey<S, Self::Entity> {
        if DAMAGE == 6 {
            KEY_EXPRESSION.with(|evaluated| {
                assert!(
                    !evaluated.replace(true),
                    "authored key expression was evaluated twice"
                )
            });
        }
        if DAMAGE == 5 {
            ApplicationAuthoredLogicalKey::from_field(wrong_codec())
        } else {
            ApplicationAuthoredLogicalKey::from_field(RawId::<false>::at(
                if matches!(DAMAGE, 3 | 6) {
                    "id_typo"
                } else {
                    "Id"
                },
            ))
        }
    }
}
impl<const DAMAGE: u8> ApplicationCorrectionBinding<S> for Correction<DAMAGE> {
    const IDENTITY: &'static str = "declaration.correction.v1";
    type Source = CreateBinding<S>;
    type InverseIntent = Retire;
    type Disclosure = query::Binding<S>;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<S, CreateOp, Create, true, false>
    {
        correction::UndoCreate::<S>::source_definition()
    }
    fn disclosure_parameters(
        original: &Create,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        correction::UndoCreate::<S>::disclosure_parameters(original)
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        if (21..=31).contains(&DAMAGE) {
            return aftermath_expressions::mechanism::<DAMAGE>();
        }
        if DAMAGE >= 10 {
            return retention::mechanism::<DAMAGE>();
        }

        if DAMAGE >= 3 {
            return DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>();
        }
        let locus = if DAMAGE == 2 {
            DeclaredPreImageLocus::from_field(wrong_codec())
        } else {
            DeclaredPreImageLocus::from_field(RawId::<false>::at(if DAMAGE == 0 {
                "id_typo"
            } else {
                "Id"
            }))
        };
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(DeclaredPreImageDemand::new(
            [locus],
            16,
        ))
    }
    fn completed_external_reconciliation() -> Option<DeclaredReconciliationProcedure> {
        aftermath_expressions::reconciliation::<DAMAGE>()
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<S>> {
        if DAMAGE == 9 {
            OUTPUT_EXPRESSION.with(|evaluated| {
                assert!(
                    !evaluated.replace(true),
                    "output expression was evaluated twice"
                )
            });
        }

        if DAMAGE < 7 {
            return Vec::new();
        }
        vec![
            ApplicationCorrectionOutput::<CreateBinding<S>, Created<S>>::field(RawId::<false>::at(
                if matches!(DAMAGE, 7 | 9) {
                    "output_typo"
                } else {
                    "Id"
                },
            )),
        ]
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<S>> {
        (matches!(DAMAGE, 3..=6)).then(
            ApplicationAuthoredCorrection::retire_created::<
                Self,
                Family<DAMAGE>,
                Created<S>,
                Retired<S>,
            >,
        )
    }
    fn derive_correction(
        original: &Create,
        material: ApplicationCorrectionSourceMaterial<'_, S>,
    ) -> Result<Retire, ApplicationCorrectionDerivationDenial> {
        correction::UndoCreate::<S>::derive_correction(original, material)
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        correction::UndoCreate::<S>::inverse_key(key)
    }
    fn visit<V: ApplicationCorrectionVisitor<S>>(
        dispatch: ApplicationCorrectionDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
// Id exists on Node under OtherFacts, and Facts exists on AnotherNode.
// The requested Node.Facts.Id is absent: both ownership axes matter.
struct OtherModel;
struct OtherNode;
struct AspectOwner;
struct OtherFacts;
impl Fragment for OtherModel {
    const NAME: &'static str = "other_model";
    type Bindings = OwnershipModelBindings;
    type Members = (OtherNode, AspectOwner);
}
impl Entity for OtherNode {
    const NAME: &'static str = "Node";
    type Aspects = (OtherFacts,);
    type Relations = ();
}
impl Entity for AspectOwner {
    const NAME: &'static str = "AnotherNode";
    type Aspects = (Facts,);
    type Relations = ();
}
impl Aspect for OtherFacts {
    const NAME: &'static str = "OtherFacts";
    type Row = FactsValue;
}
struct CodecModel;
struct Codecs;
impl Fragment for CodecModel {
    const NAME: &'static str = "codec_model";
    type Bindings = Codecs;
    type Members = (NativeNode<false>,);
}
impl BindingSet for Codecs {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for Codecs {
    type Binding = OtherU64;
}
impl Record<FactsValue> for Codecs {
    fn record() -> impl Shape<FactsValue, Self> {
        field("Id", |row: &FactsValue| row.id).build(|id, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(FactsValue { id })
        })
    }
}
fn wrong_codec() -> FieldSelection<CodecModel, NativeNode<false>, Facts, u64> {
    assert_eq!(
        OtherU64::SCALAR_FAMILY,
        U64ApplicationValueBinding::SCALAR_FAMILY
    );
    assert_ne!(OtherU64::IDENTITY, U64ApplicationValueBinding::IDENTITY);
    FieldSelection::at("Id")
}
// Same u64 family and implementation, with a different portable codec identity.
struct OtherU64;
impl ApplicationScalarValueBinding for OtherU64 {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "declaration.other.u64";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        U64ApplicationValueBinding::SCALAR_FAMILY;
    const SUPPORTS_ADMITTED_DECODE: bool = U64ApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        U64ApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &u64) -> Result<(), ApplicationValueValidationDenial> {
        U64ApplicationValueBinding::validate(value)
    }
    fn encode(
        value: &u64,
    ) -> Result<worth_foundational::facade::AspectValue, ApplicationValueEncodeDenial> {
        U64ApplicationValueBinding::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for OtherU64 {
    fn decode(
        value: &worth_foundational::facade::AspectValue,
    ) -> Result<u64, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode(value)
    }
    fn decode_admitted(
        value: &worth_foundational::facade::AspectValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<u64, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode_admitted(value, admission)
    }
}
impl ApplicationIdentityScalarValueBinding for OtherU64 {}

fn refused<const DAMAGE: u8>(foreign_aspect: bool, path: &'static str) {
    // Expression construction, the infallible hook and fragment registration all
    // succeed. Only the builder that composes actual schema membership refuses it.
    let _ = Correction::<DAMAGE>::mechanism();
    let builder = if foreign_aspect {
        ApplicationSchemaDeclarationBuilder::<S>::from_fragment(OtherModel).unwrap()
    } else {
        ApplicationSchemaDeclarationBuilder::<S>::from_fragment(OwnershipModel::<false>).unwrap()
    }
    .correction::<Correction<DAMAGE>>();
    let denial = builder.build().unwrap_err();
    let field = RawId::<false>::at(path).binding_recipe().locus().clone();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidCorrectionField {
            correction: "declaration.correction.v1".to_owned(),
            field: field.clone(),
        }
    );
    let rendered = denial.to_string();
    for expected in [
        "declaration.correction.v1",
        field.entity(),
        field.aspect(),
        field.field(),
    ] {
        assert!(
            rendered.contains(expected),
            "{rendered} must name {expected}"
        );
    }
}
#[test]
fn a_misspelled_preimage_field_is_refused_at_schema_composition() {
    refused::<0>(false, "id_typo");
}
#[test]
fn a_preimage_aspect_outside_its_entity_is_refused_at_schema_composition() {
    refused::<1>(true, "Id");
}
#[test]
fn a_preimage_codec_mismatch_is_refused_at_schema_composition() {
    refused::<2>(false, "Id");
}
#[test]
fn a_misspelled_authored_key_is_refused_at_schema_composition() {
    refused::<3>(false, "id_typo");
}
#[test]
fn an_authored_key_aspect_outside_its_entity_is_refused_at_schema_composition() {
    refused::<4>(true, "Id");
}
#[test]
fn an_authored_key_codec_mismatch_is_refused_at_schema_composition() {
    refused::<5>(false, "Id");
}

thread_local! {
    static KEY_EXPRESSION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
struct KeyExpressionScope;
impl Drop for KeyExpressionScope {
    fn drop(&mut self) {
        KEY_EXPRESSION.with(|evaluated| evaluated.set(false));
    }
}
#[test]
fn composition_refuses_one_authored_key_expression_without_evaluating_it_again() {
    let _scope = KeyExpressionScope;
    refused::<6>(false, "id_typo");
}

#[test]
fn a_misspelled_output_field_is_refused_at_schema_composition() {
    refused::<7>(false, "output_typo");
}
#[test]
fn an_output_aspect_outside_its_entity_is_refused_at_schema_composition() {
    refused::<8>(true, "Id");
}

thread_local! { static OUTPUT_EXPRESSION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[test]
fn composition_checks_one_output_expression_without_evaluating_it_again() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            OUTPUT_EXPRESSION.with(|value| value.set(false));
        }
    }
    let _reset = Reset;
    refused::<9>(false, "output_typo");
}

mod retention;

mod aftermath_expressions;

fn compose<const DAMAGE: u8>(
) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
    ApplicationSchemaDeclarationBuilder::<S>::from_fragment(OwnershipModel::<false>)
        .unwrap()
        .operation(
            Correction::<DAMAGE>::source_definition()
                .aftermath(DeclaredApplicationAftermathContract::correction_binding::<
                    Correction<DAMAGE>,
                >())
                .finish(),
        )
        .build()
}
