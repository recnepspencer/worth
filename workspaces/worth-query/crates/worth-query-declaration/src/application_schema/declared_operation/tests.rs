//! The same typed facts grant capabilities and populate the ordinary schema.
use crate::application_schema::*;
mod generic;

use crate::application_schema::fragment::{self, FieldSelection};
pub(super) struct Schema;
impl fragment::Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = fragment::NoBindings;
    type Members = (AccessModel, Contracts);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.test.declared-operation";
    const NAME: &'static str = <Self as fragment::Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub(super) struct AccessModel;
impl fragment::Fragment for AccessModel {
    const NAME: &'static str = "access_model";
    type Bindings = AccessBindings;
    type Members = (Record,);
}
pub(super) struct AccessBindings;
impl fragment::BindingSet for AccessBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for AccessBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub(super) struct Record;
impl fragment::Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = (Facts,);
    type Relations = (Children,);
}
pub(super) struct Facts;
impl fragment::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = AccessRow;
}
pub(super) struct AccessRow {
    value: u64,
    inherited: u64,
}
impl fragment::Record<AccessRow> for AccessBindings {
    fn record() -> impl fragment::Shape<AccessRow, Self> {
        fragment::field("Value", |row: &AccessRow| row.value)
            .and("InheritedValue", |row: &AccessRow| row.inherited)
            .build(|value, inherited, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccessRow { value, inherited })
            })
    }
}
pub(super) type Value = FieldSelection<AccessModel, Record, Facts, u64>;
pub(super) type InheritedValue = FieldSelection<AccessModel, Record, Facts, u64>;
pub(super) struct Children;
impl fragment::Relation for Children {
    const NAME: &'static str = "Children";
    type To = Record;
}
impl Children {
    fn reference() -> ApplicationRelationRef<Schema, Self, Record, Record> {
        fragment::relation(AccessModel, Record, Self)
    }
}
pub struct Contracts;
impl fragment::Registration<Schema> for Contracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .declared_operation::<Edit>(
                Edit::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .declared_operation::<generic::GenericEdit>(
                generic::GenericEdit::reference::<Schema>()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            ))
    }
}
crate::worth_query_structured_value_binding!(Input for () {
    identity: "worth.test.declared-operation.input.v1"
});
crate::worth_query_operation!(Edit for Schema, input Input;
    access {
        read_entities: [Record];
        read_fields: [Value::at("Value")];
        read_relations: [Children];
        writes: [Value::at("Value")];
        expects_version: [Value::at("Value")];
        expects_fact: [Value::at("Value")];
        creates: [Record];
        deletes: [Record];
        links: [Children];
        unlinks: [Children];
    }
);

#[test]
fn declared_access_supplies_capabilities_and_schema_without_an_invoice() {
    fn accepts_field<F: fragment::RecordSelection<Value = u64>>(_: F) {}
    fn accepts_entity<T: OperationReads<Edit> + OperationCreates<Edit> + OperationDeletes<Edit>>() {
    }
    fn accepts_relation<T: OperationReads<Edit> + OperationLinks<Edit> + OperationUnlinks<Edit>>() {
    }
    accepts_field(Value::at("Value"));
    accepts_entity::<Record>();
    accepts_relation::<Children>();

    let declaration = Schema::declaration().expect("ordinary generated access is valid");
    let members = declaration.erased().members();
    assert_declared_reads(members);
    assert_declared_effects(members);
    assert_declared_expectations(members);
    assert!(!members.iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::OperationProjectionWorkBudget { .. }
    )));
}

fn assert_declared_reads(members: &[ApplicationSchemaMember]) {
    for expected in [
        ApplicationOperationDecisionReadTarget::Entity {
            entity: "Record".into(),
        },
        ApplicationOperationDecisionReadTarget::Field {
            entity: "Record".into(),
            aspect: fragment::aspect(AccessModel, Record, Facts).name().into(),
            field: "Value".into(),
        },
        ApplicationOperationDecisionReadTarget::Relation {
            relation: "Children".into(),
            from: "Record".into(),
            to: "Record".into(),
        },
    ] {
        assert!(members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead { operation, target }
            if operation == "Edit" && target == &expected)));
    }
}

fn assert_declared_effects(members: &[ApplicationSchemaMember]) {
    for expected in [
        ApplicationOperationProgramTarget::Write {
            entity: "Record".into(),
            aspect: fragment::aspect(AccessModel, Record, Facts).name().into(),
            field: "Value".into(),
        },
        ApplicationOperationProgramTarget::Create {
            entity: "Record".into(),
        },
        ApplicationOperationProgramTarget::Delete {
            entity: "Record".into(),
        },
        ApplicationOperationProgramTarget::Link {
            relation: "Children".into(),
            from: "Record".into(),
            to: "Record".into(),
        },
        ApplicationOperationProgramTarget::Unlink {
            relation: "Children".into(),
            from: "Record".into(),
            to: "Record".into(),
        },
    ] {
        assert!(members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationProgram { operation, target }
            if operation == "Edit" && target == &expected)));
    }
}

fn assert_declared_expectations(members: &[ApplicationSchemaMember]) {
    for expected in [
        ApplicationMutationPreconditionFamily::ExpectedVersion,
        ApplicationMutationPreconditionFamily::ExpectedFact,
    ] {
        assert!(members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationMutationPrecondition { operation, target }
            if operation == "Edit" && target.family() == expected)));
    }
}
