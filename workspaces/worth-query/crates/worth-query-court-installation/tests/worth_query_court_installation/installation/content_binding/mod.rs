//! Typed content recipes cross the real Decl-to-Installation boundary.
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledApplicationSchemaDenialKind as Kind, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub(crate) mod codec;
use codec::{Good, Impostor};
trait Model: ApplicationSchema {}
struct Schema<const MODE: u8>;
impl<const MODE: u8> Model for Schema<MODE> {}
impl<const MODE: u8> ApplicationSchema for Schema<MODE> {
    const OWNER: &'static str = "content.installation.tests";
    const NAME: &'static str = "ContentSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let builder = ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(Entity::reference())
            .aspect(Entity::reference(), Samples::reference());
        let builder = if MODE == 2 {
            builder
        } else {
            builder.content_format::<Good>()
        };
        let builder = if MODE == 1 {
            builder.content_field(
                Entity::reference(),
                content_field(Values::reference(), Impostor),
            )
        } else {
            builder.content_field(
                Entity::reference(),
                content_field(Values::reference(), Good),
            )
        };
        builder.build()
    }
}
worth_query_declaration::worth_query_entity!(Entity for S:Model);
worth_query_declaration::worth_query_aspect!(Samples for S:Model,Entity; identity=AspectIdentity(3131),revision=AspectContractRevision(1));
worth_query_declaration::worth_query_field!(Values for S:Model,Entity,Samples:ContentRefId=>ContentRefApplicationValueBinding,read_write,no_equality);
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
    .application_schema(declaration.clone())
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
        .field("Entity", "Samples", "Values")
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
fn content_equal_descriptors_do_not_certify_a_foreign_field_binding_type() {
    let denied = match bind::<1>() {
        Ok(_) => panic!("a foreign typed field recipe installed"),
        Err(cause) => cause,
    };
    assert_eq!(denied.kind(), Kind::ContentFormatMismatch);
}
#[test]
fn content_reference_without_a_declared_format_opens_no_installation_door() {
    let denied = Schema::<2>::declaration().unwrap_err();
    assert_eq!(
        denied,
        ApplicationSchemaDeclarationDenial::MissingContentFormat
    );
}
