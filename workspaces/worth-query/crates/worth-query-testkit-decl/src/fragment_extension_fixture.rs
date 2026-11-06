//! Separate features add optional facts to the shared entity without reversing ownership.
use super::fragment_auth_fixture::ChangeForm;
use super::fragment_fixture::*;
pub struct Annotations;
pub struct Classifications;
pub struct Annotation;
pub struct Classified;
pub struct Classification;
impl Fragment for Annotations {
    const NAME: &'static str = "annotations";
    type Bindings = Bindings;
    type Members = ((Extend<Item, (Option<Annotation>,)>,), ExtensionAccess);
}
impl Fragment for Classifications {
    const NAME: &'static str = "classifications";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Option<Exclusive<Classification>>,)>,);
}
impl Aspect for Annotation {
    const NAME: &'static str = "annotation";
    type Row = OptionalText;
}
impl Aspect for Classified {
    const NAME: &'static str = "classified";
    type Row = Caption;
}
impl OneOf for Classification {
    type Row = Caption;
    type Cases = (Classified,);
}
pub struct ExtensionAccess;
impl Registration<Fixture> for ExtensionAccess {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        let operation = ApplicationOperationRef::<Fixture, ChangeForm, ()>::from_declaration();
        Ok(builder
            .operation_read_record(operation, Annotations, Item, Annotation)
            .operation_write_record(operation, Annotations, Item, Annotation)
            .operation_read_record(operation, Classifications, Item, Classification)
            .operation_write_record(operation, Classifications, Item, Classified))
    }
}
