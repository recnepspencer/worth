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
    type Members = (
        (
            Extend<
                Item,
                (
                    Option<Annotation>,
                    Option<super::fragment_codec_fixture::ExtendedValue>,
                ),
                (WithAspects<Annotates, (Option<EdgeWeight>,)>,),
            >,
        ),
        ExtensionAccess,
    );
}
impl Fragment for Classifications {
    const NAME: &'static str = "classifications";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Option<Exclusive<Classification>>,), (ClassifiedBy,)>,);
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
            .operation_read_record(
                operation,
                Fixture,
                Item,
                super::fragment_codec_fixture::OwnedValue,
            )
            .operation_read_record(
                operation,
                Annotations,
                Item,
                super::fragment_codec_fixture::ExtendedValue,
            )
            .operation_read_relation(operation, relation(Annotations, Item, Annotates))
            .operation_link(operation, relation(Annotations, Item, Annotates))
            .operation_read_record(operation, Annotations, Item, Annotation)
            .operation_write_record(operation, Annotations, Item, Annotation)
            .operation_read_record(operation, Classifications, Item, Classification)
            .operation_write_record(operation, Classifications, Item, Classified))
    }
}

pub struct Annotates;
impl Relation for Annotates {
    const NAME: &'static str = "annotates";
    type To = Item;
}
pub struct ClassifiedBy;
impl Relation for ClassifiedBy {
    const NAME: &'static str = "classified_by";
    type To = Item;
}
pub struct EdgeWeight;
impl Aspect for EdgeWeight {
    const NAME: &'static str = "weight";
    type Row = EdgeWeightRow;
}

impl OperationReads<ChangeForm> for Annotates {}
impl OperationLinks<ChangeForm> for Annotates {}

#[derive(Debug, Eq, PartialEq)]
pub struct EdgeWeightRow(pub u64);
impl Record<EdgeWeightRow> for Bindings {
    fn record() -> impl Shape<EdgeWeightRow, Self> {
        field("value", |row: &EdgeWeightRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EdgeWeightRow(value))
            })
    }
}
