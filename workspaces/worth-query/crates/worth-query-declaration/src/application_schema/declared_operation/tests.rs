//! The same typed facts grant capabilities and populate the ordinary schema.
use crate::application_schema::*;
mod generic;

crate::worth_query_application_schema! {
    schema Schema {
        owner: "worth.test.declared-operation",
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Record::reference())
                .aspect(Record::reference(), Facts::reference())
                .field(Record::reference(), Value::reference())
                .field(Record::reference(), generic::InheritedValue::reference())
                .relation(Children::reference(), Record::reference(), Record::reference())
                .declared_operation::<Edit>(Edit::reference().definition()
                    .no_external_effect().no_aftermath().finish())
                .declared_operation::<generic::GenericEdit>(generic::GenericEdit::reference::<Schema>().definition()
                    .no_external_effect().no_aftermath().finish())
        }
    }
}
crate::worth_query_entity!(Record for S: generic::Bound);
crate::worth_query_aspect!(Facts for S: generic::Bound, Record;
    identity = AspectIdentity(0x9161_5f01), revision = AspectContractRevision(1),);
crate::worth_query_field!(Value for S: generic::Bound, Record, Facts:
    u64 => U64ApplicationValueBinding, read_write, equality);
crate::worth_query_relation!(Children in Schema, Record => Record;
    integrity = same_context_unbounded_retain_dangling);
crate::worth_query_structured_value_binding!(Input for () {
    identity: "worth.test.declared-operation.input.v1"
});
crate::worth_query_operation!(Edit for Schema, input Input;
    access {
        read_entities: [Record];
        read_fields: [Value];
        read_relations: [Children];
        writes: [Value];
        expects_version: [Value];
        expects_fact: [Value];
        creates: [Record];
        deletes: [Record];
        links: [Children];
        unlinks: [Children];
    }
);

#[test]
fn declared_access_supplies_capabilities_and_schema_without_an_invoice() {
    fn accepts_field<
        T: OperationReads<Edit>
            + OperationWrites<Edit>
            + OperationExpectsVersion<Edit>
            + OperationExpectsFact<Edit>,
    >() {
    }
    fn accepts_entity<T: OperationReads<Edit> + OperationCreates<Edit> + OperationDeletes<Edit>>() {
    }
    fn accepts_relation<T: OperationReads<Edit> + OperationLinks<Edit> + OperationUnlinks<Edit>>() {
    }
    accepts_field::<Value>();
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
            aspect: "Facts".into(),
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
            aspect: "Facts".into(),
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
