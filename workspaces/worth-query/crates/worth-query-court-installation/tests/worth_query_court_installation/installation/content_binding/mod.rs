//! Typed content recipes cross the real Decl-to-Installation boundary.
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationSchemaDenial, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub(crate) mod codec;
use codec::{Good, Impostor};
struct Schema<const MODE: u8>;
impl<const MODE: u8> ApplicationSchema for Schema<MODE> {
    const OWNER: &'static str = "content.installation.tests";
    const NAME: &'static str = "ContentSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(ContentRecords::<MODE>)?.build()
    }
}
use worth_query_declaration::facade::application_schema::fragment::{self, *};
struct ContentRecords<const MODE: u8>;
impl<const MODE: u8> Fragment for ContentRecords<MODE> {
    const NAME: &'static str = "content_records";
    type Bindings = ContentBindings<MODE>;
    type Members = (Entity,);
}
struct ContentBindings<const MODE: u8>;
impl<const MODE: u8> BindingSet for ContentBindings<MODE> {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl<const MODE: u8> Scalar<ContentRefId> for ContentBindings<MODE> {
    type Binding = ContentRefApplicationValueBinding;
    fn content_format() -> Option<ApplicationContentBindingRecipe> {
        if MODE == 2 {
            None
        } else if MODE == 1 {
            Some(ApplicationContentBindingRecipe::of::<Impostor>())
        } else {
            Some(ApplicationContentBindingRecipe::of::<Good>())
        }
    }
}
struct Entity;
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Samples,);
    type Relations = ();
}
struct Samples;
impl Aspect for Samples {
    const NAME: &'static str = "Samples";
    type Row = SampleValue;
}
struct SampleValue(ContentRefId);
impl<const MODE: u8> fragment::Record<SampleValue> for ContentBindings<MODE> {
    fn record() -> impl Shape<SampleValue, Self> {
        field("Values", |row: &SampleValue| row.0).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SampleValue(value))
            },
        )
    }
}
type Values = FieldSelection<ContentRecords<0>, Entity, Samples, ContentRefId>;
fn bind<const MODE: u8>() -> Result<
    WorthQueryInstalledApplicationSchema<Schema<MODE>>,
    WorthQueryInstalledApplicationSchemaDenial,
> {
    let declaration = Schema::<MODE>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::<MODE>::OWNER,
        1,
        0,
    ))
    .application_schema(Schema::<0>::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
}
#[test]
fn content_installed_recipe_matches_the_actual_field_type_and_full_meaning_alias() {
    let installed = bind::<0>().unwrap();
    let binding = installed
        .value_bindings()
        .field("Entity", Values::at("Values").aspect(), "Values")
        .unwrap();
    assert!(binding.matches_content::<Good>());
    assert!(!binding.matches_content::<Impostor>());
    assert_eq!(
        binding.content_format().unwrap().declaration().identity,
        [31; 32]
    );
    let field = installed
        .installed_declaration()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::Field { .. }))
        .unwrap();
    let meaning = ApplicationFieldValueMeaning::from_member(field)
        .unwrap()
        .unwrap();
    assert_eq!(binding.content_codec(), Some(meaning.semantic_digest()));
    assert_ne!(binding.content_codec(), Some(Good::IDENTITY));
}
#[test]
fn content_binding_set_owns_its_codec_even_when_descriptors_are_equal() {
    let installed = bind::<1>().unwrap();
    let binding = installed
        .value_bindings()
        .field("Entity", Values::at("Values").aspect(), "Values")
        .unwrap();
    assert!(binding.matches_content::<Impostor>());
    assert!(!binding.matches_content::<Good>());
}
#[test]
fn content_reference_without_a_declared_format_opens_no_installation_door() {
    let declaration = Schema::<0>::declaration().unwrap();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
    parts
        .members
        .retain(|member| !matches!(member, ApplicationSchemaMember::ContentFormat { .. }));
    let denied = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap_err();
    assert_eq!(
        denied,
        ApplicationSchemaDeclarationDenial::MissingContentFormat
    );
}
