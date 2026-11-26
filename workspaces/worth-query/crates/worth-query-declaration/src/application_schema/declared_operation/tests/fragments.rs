//! Included access fragments lower as inline clauses would, and the access
//! they repeat registers once.
use super::*;

mod posting {
    crate::worth_query_access_fragment! {
        /// Every posting reads the record and reads and writes its value.
        value_posting {
            read_entities: [crate::application_schema::declared_operation::tests::Record];
            read_fields: [crate::application_schema::declared_operation::tests::Value::at("Value")];
            writes: [crate::application_schema::declared_operation::tests::Value::at("Value")];
        }
    }
    pub(crate) use value_posting;

    crate::worth_query_access_fragment! {
        /// A nested include plus an expectation on the value it already reads.
        guarded_posting {
            include: [crate::application_schema::declared_operation::tests::fragments::posting::value_posting];
            expects_version: [crate::application_schema::declared_operation::tests::Value::at("Value")];
        }
    }
    pub(crate) use guarded_posting;

    crate::worth_query_access_fragment! {
        /// Overlaps `value_posting` on its field read and write.
        value_audit {
            read_fields: [
                crate::application_schema::declared_operation::tests::Value::at("Value"),
                crate::application_schema::declared_operation::tests::InheritedValue::at("InheritedValue"),
            ];
            writes: [crate::application_schema::declared_operation::tests::Value::at("Value")];
            links: [crate::application_schema::declared_operation::tests::Children];
        }
    }
    pub(crate) use value_audit;
}

mod moves {
    use super::super::{generic::Bound, Input, Record, Schema, Value};
    crate::worth_query_operation!(pub(in super::super) Restock for Schema, input Input;
        access {
            include: [super::posting::guarded_posting, super::posting::value_audit];
            read_fields: [Value::at("Value")];
            creates: [Record];
        }
    );
    crate::worth_query_operation!(pub(in super::super) GenericRestock for S: Bound, input Input;
        access { include: [super::posting::value_posting]; }
    );
}
pub(super) use moves::{GenericRestock, Restock};

fn count(members: &[ApplicationSchemaMember], operation: &str, field: &str) -> [usize; 3] {
    let named = |name: &str, entity: &str| name == operation && entity == "Record";
    let reads = members.iter().filter(|member| matches!(member,
        ApplicationSchemaMember::OperationDecisionRead {
            operation, target: ApplicationOperationDecisionReadTarget::Field { entity, field: f, .. },
        } if named(operation, entity) && f == field));
    let writes = members.iter().filter(|member| {
        matches!(member,
        ApplicationSchemaMember::OperationProgram {
            operation, target: ApplicationOperationProgramTarget::Write { entity, field: f, .. },
        } if named(operation, entity) && f == field)
    });
    let expectations = members.iter().filter(|member| {
        matches!(member,
        ApplicationSchemaMember::OperationMutationPrecondition { operation: name, target }
        if name == operation && target.field_name() == field)
    });
    [reads.count(), writes.count(), expectations.count()]
}

#[test]
fn included_fragments_grant_their_capabilities_and_repeated_access_registers_once() {
    fn reads<T: OperationReads<Restock>>() {}
    fn creates<T: OperationCreates<Restock>>() {}
    fn links<T: OperationLinks<Restock>>() {}
    fn generic_reads<T: OperationReads<GenericRestock>>() {}
    reads::<Record>();
    creates::<Record>();
    links::<Children>();
    generic_reads::<Record>();

    let declaration = Schema::declaration().expect("overlapping fragments declare one access set");
    let members = declaration.erased().members();
    // `Value` is read by both fragments and inline, written by both, and
    // expected once by the nested fragment: each registers exactly once.
    assert_eq!(count(members, "Restock", "Value"), [1, 1, 1]);
    assert_eq!(count(members, "Restock", "InheritedValue"), [1, 0, 0]);
    assert_eq!(count(members, "GenericRestock", "Value"), [1, 1, 0]);
    assert!(members.iter().any(|member| matches!(member,
        ApplicationSchemaMember::OperationProgram {
            operation, target: ApplicationOperationProgramTarget::Link { relation, .. },
        } if operation == "Restock" && relation == "Children")));
}
