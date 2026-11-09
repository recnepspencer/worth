//! Owner-local portable readmission uses real validated and installed contracts.
use super::*;
use crate::facade::*;
use worth_query_declaration::facade::application_schema::fragment as records;
use worth_query_declaration::facade::application_schema::*;
struct Schema;
impl records::Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = Bindings;
    type Members = (Entity, Remove);
}
struct Bindings;
impl records::BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl records::Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
struct Entity;
impl records::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Option<Input>,);
    type Relations = ();
}
struct Input;
impl records::Aspect for Input {
    const NAME: &'static str = "Input";
    type Row = Pair;
}
impl Input {
    fn reference() -> records::AspectSelection<Schema, Entity, Self> {
        records::aspect(Schema, Entity, Self)
    }
}
struct Pair {
    left: u64,
    right: u64,
}
impl records::Record<Pair> for Bindings {
    fn record() -> impl records::Shape<Pair, Self> {
        records::field("Left", |r: &Pair| r.left)
            .equality_queryable(true)
            .and("Right", |r: &Pair| r.right)
            .equality_queryable(true)
            .build(|left, right, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Pair { left, right })
            })
    }
}
type Left = records::FieldSelection<Schema, Entity, Input, u64>;
type Right = records::FieldSelection<Schema, Entity, Input, u64>;
worth_query_declaration::worth_query_structured_value_binding!(InputBinding for () { identity: "worth.rust.unit" });
worth_query_declaration::worth_query_operation!(Remove for Schema, input InputBinding);
worth_query_declaration::worth_query_operation_removes_aspects!(Remove => [Input]);
impl records::Registration<Schema> for Remove {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(Self::reference(), Left::at("Left"))
            .operation_read_field(Self::reference(), Right::at("Right"))
            .operation_remove_aspect(Self::reference(), Input::reference()))
    }
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "removal-court";
    const NAME: &'static str = <Self as records::Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        records::declare(Self)
    }
}
fn installed() -> WorthQueryInstalledApplicationSchema<Schema> {
    let declaration = Schema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "removal-court",
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
    .unwrap()
}
#[test]
fn installed_parent_removal_readmission_requires_complete_same_binding_native_reads() {
    let schema = installed();
    let operation = schema.installed_operation(Remove::reference()).unwrap();
    let binding = schema.binding_identity();
    let contract = schema
        .native_contracts()
        .aspect("Entity", Input::reference().name())
        .unwrap()
        .contract()
        .clone();
    let portable = [
        crate::package::WorthQueryPortableOperationTouchScope::RemoveAspect {
            schema: "Schema".into(),
            entity: "Entity".into(),
            contract,
        },
    ];
    let reads = operation.contracts().graph_reads();
    let (touches, _) =
        install_portable_graph_touches(&binding, schema.native_contracts(), &portable, reads)
            .unwrap();
    assert!(
        matches!(&touches.scopes()[0], WorthQueryOperationTouchScope::RemoveAspect(scope) if scope.schema() == &binding)
    );
    let missing = crate::domain_operation::WorthQueryOperationGraphReadContract::NotRequired;
    assert!(install_portable_graph_touches(
        &binding,
        schema.native_contracts(),
        &portable,
        &missing
    )
    .is_err());
    let foreign = installed();
    let foreign_operation = foreign.installed_operation(Remove::reference()).unwrap();
    assert_ne!(foreign.binding_identity(), binding);
    assert!(install_portable_graph_touches(
        &binding,
        schema.native_contracts(),
        &portable,
        foreign_operation.contracts().graph_reads()
    )
    .is_err());
}
