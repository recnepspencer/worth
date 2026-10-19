use std::any::TypeId;
use std::collections::{BTreeSet, HashSet};

use super::super::program_support_fixture::{validated, BoundedProgram};
use super::WorthQueryProgramSupportEntry;

#[test]
fn support_answers_only_declared_operation_and_binding_membership() {
    struct DeclaredOperation;
    struct DeclaredBinding;
    struct ConditionalBinding;
    struct UndeclaredOperation;
    struct UndeclaredBinding;

    let program = validated::<BoundedProgram>();
    let entry = WorthQueryProgramSupportEntry::admitted(
        *program.revision(),
        program.identity().clone(),
        BTreeSet::new(),
        HashSet::from([TypeId::of::<DeclaredOperation>()]),
        HashSet::from([
            TypeId::of::<DeclaredBinding>(),
            TypeId::of::<ConditionalBinding>(),
        ]),
        HashSet::from([TypeId::of::<ConditionalBinding>()]),
        program.semantic_description().clone(),
        BTreeSet::new(),
        BTreeSet::new(),
    );

    assert!(entry.acts_through_operation(TypeId::of::<DeclaredOperation>()));
    assert!(!entry.acts_through_operation(TypeId::of::<UndeclaredOperation>()));
    assert!(entry.acts_through_mutation_binding(TypeId::of::<DeclaredBinding>()));
    assert!(!entry.acts_through_mutation_binding(TypeId::of::<UndeclaredBinding>()));
    assert!(entry.declares_mutation_binding(TypeId::of::<ConditionalBinding>()));
    assert!(!entry.acts_through_mutation_binding(TypeId::of::<ConditionalBinding>()));
}
