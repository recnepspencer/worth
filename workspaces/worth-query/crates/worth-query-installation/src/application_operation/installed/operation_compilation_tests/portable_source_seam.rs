//! Split-source courts for the retained portable operation spine.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
    DeclaredCorrectionMechanism, DeclaredReconciliationProcedure,
};

use super::*;

#[test]
fn retained_portable_reads_are_the_aftermath_coverage_source() {
    let portable_members = members("freeze", "freeze", 64);
    let mut divergent_summaries = portable_members.clone();
    for member in &mut divergent_summaries {
        if let ApplicationSchemaMember::OperationDecisionRead { operation, .. } = member {
            *operation = "sibling".to_owned();
        }
    }

    let installed = compile_with_portable_members(&divergent_summaries, &portable_members)
        .expect("retained portable read covers the inverse despite divergent summaries");
    assert!(installed.aftermath().is_some());
}

#[test]
fn retained_portable_program_width_is_the_resource_source() {
    let portable_members = members("freeze", "freeze", 64);
    let mut wider_summaries = portable_members.clone();
    wider_summaries.push(ApplicationSchemaMember::OperationProgram {
        operation: "freeze".to_owned(),
        target: ApplicationOperationProgramTarget::Delete {
            entity: "Audit".to_owned(),
        },
    });

    let baseline = compile_with_portable_members(&portable_members, &portable_members).unwrap();
    let divergent = compile_with_portable_members(&wider_summaries, &portable_members).unwrap();
    assert_eq!(baseline.resources(), divergent.resources());
}

#[test]
fn retained_portable_reconciliation_is_the_installed_source() {
    let portable_members = external_owner_members("portable-confirmation");
    let divergent_summaries = external_owner_members("member-summary-confirmation");

    let installed = compile_with_portable_members(&divergent_summaries, &portable_members).unwrap();
    assert_eq!(
        installed
            .aftermath()
            .and_then(|aftermath| aftermath.reconciliation())
            .map(|procedure| procedure.procedure_slot()),
        Some("portable-confirmation")
    );
}

fn external_owner_members(reconciliation: &str) -> Vec<ApplicationSchemaMember> {
    vec![
        operation("freeze"),
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Create {
                entity: "Audit".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "freeze".to_owned(),
            maximum_work_units: 16,
        },
        external_owner_aftermath_member("freeze", reconciliation),
    ]
}

fn external_owner_aftermath_member(
    operation: &'static str,
    reconciliation: &str,
) -> ApplicationSchemaMember {
    let compensation = DeclaredCompensation::new(
        "compensate-freeze",
        DeclaredAftermathPostcondition::BusinessPostcondition {
            identity: "compensated".into(),
        },
    )
    .unwrap();
    let contract = DeclaredApplicationAftermathContract::runtime_with_external_owner(
        DeclaredCorrectionMechanism::Compensation(compensation),
        DeclaredReconciliationProcedure::new(reconciliation).unwrap(),
    );
    let definition =
        ApplicationOperationRef::<Schema, CompilationOperation, ()>::from_declaration()
            .definition()
            .no_external_effect()
            .aftermath(contract)
            .finish();
    let declaration = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(definition)
        .build()
        .unwrap();
    let mut member = declaration
        .erased()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::OperationAftermath { .. }))
        .unwrap()
        .clone();
    let ApplicationSchemaMember::OperationAftermath {
        operation: installed,
        ..
    } = &mut member
    else {
        unreachable!()
    };
    *installed = operation.to_owned();
    member
}
