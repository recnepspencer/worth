//! Cold records cannot drop native effect/read closure or alter trigger coupling.
use super::super::program;
use super::super::schema::{Document, SetRetention};
use worth_query_decl::facade::{
    worth_query_operation_deletes, worth_query_operation_reads, worth_query_operation_unlinks,
};
use worth_query_host::facade::declaration::application_schema::*;
struct AliasingSelection;
worth_query_operation_reads!(SetRetention => [AliasingSelection]);
worth_query_operation_unlinks!(SetRetention => [AliasingSelection]);
worth_query_operation_deletes!(SetRetention => [Document]);

#[test]
fn typed_finish_rejects_a_structurally_valid_selection_that_would_delete_the_root() {
    use super::{super::schema::*, declaration::*};
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let relation = ApplicationRelationRef::<
        DocumentRetentionSchema<120>,
        AliasingSelection,
        Audit,
        Document,
    >::from_schema_identifiers(
        "AliasingSelection",
        "Audit",
        "Document",
        ApplicationRelationIntegrity::new(
            base.endpoints,
            ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, Some(1)),
            base.deletion,
        ),
    );
    let result = SetRetention::reference::<DocumentRetentionSchema<120>>()
        .body()
        .assign_input(DocumentRetentionField::reference(), "retention_days")
        .unwrap()
        .retire_owned_selections_on_input_change(
            DocumentRetentionField::reference(),
            "retention_days",
            OwnedRows::reference(),
            relation,
            std::num::NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
        .finish();
    assert!(matches!(
        result,
        Err(ApplicationOperationBodyAuthoringDenial::InvalidSelectionRetirement)
    ));
}
#[test]
fn fresh_readmission_reconstructs_retirement_meaning_and_required_authority() {
    let schema = program::schema::<120>();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::project(schema.erased())
        )
        .unwrap(),
        *schema.erased()
    );
    for corrupt in ["unlink", "read", "member", "alias", "duplicate", "signing"] {
        let mut parts =
            WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
        if matches!(corrupt, "unlink" | "read") {
            let mut replaced = 0;
            for member in &mut parts.members {
                match member {
                    ApplicationSchemaMember::OperationProgram {
                        operation,
                        target: ApplicationOperationProgramTarget::Unlink { relation, .. },
                    } if corrupt == "unlink"
                        && operation == "SetRetention"
                        && relation == "CurrentSelection" =>
                    {
                        // Preserve contribution ordinals and all independent schema validity.
                        *relation = "AlternateSelection".into();
                        replaced += 1;
                    }
                    ApplicationSchemaMember::OperationDecisionRead { operation, target }
                        if corrupt == "read"
                            && operation == "SetRetention"
                            && matches!(&*target, ApplicationOperationDecisionReadTarget::Relation { relation, .. } if relation == "OwnedRows") =>
                    {
                        *target = ApplicationOperationDecisionReadTarget::Relation {
                            relation: "AlternateSelection".into(),
                            from: "Audit".into(),
                            to: "Snapshot".into(),
                        };
                        replaced += 1;
                    }
                    _ => {}
                }
            }
            assert_eq!(replaced, 1);
        } else {
            let mut found = false;
            for member in &mut parts.members {
                if let ApplicationSchemaMember::OperationBody { operation, body } = member
                    && operation == "SetRetention"
                {
                    let mut rules = body.retirements().to_vec();
                    match corrupt {
                        "member" => rules[0].member = "identity".into(),
                        "alias" => rules[0].selected = "Document".into(),
                        "duplicate" => rules.push(rules[0].clone()),
                        "signing" => {
                            // Descriptive negative only: no authentication event or signature is minted.
                            let creation =
                                body.creation()
                                    .unwrap()
                                    .clone()
                                    .with_untrusted_signing(Some(
                                        ApplicationOperationBodySigningSpec::from_untrusted(
                                            "approval".into(),
                                            ApplicationFieldBindingLocus::new(
                                                "Audit", "Facts", "Actor",
                                            ),
                                            ApplicationFieldBindingLocus::new(
                                                "Audit",
                                                "Facts",
                                                "Provenance",
                                            ),
                                        ),
                                    ));
                            *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                                "Document".into(),
                                body.scope_fields().to_vec(),
                                Some(creation),
                                vec![],
                            );
                        }
                        _ => unreachable!(),
                    }
                    *body = body.clone().with_untrusted_retirements(rules);
                    found = true;
                }
            }
            assert!(found);
        }
        parts.members.sort();
        let denied = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        let denied_operation = match &denied {
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency {
                operation,
                ..
            } if corrupt == "read" => operation,
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency {
                operation,
                ..
            } if corrupt != "read" => operation,
            other => panic!("{corrupt}: {other:?}"),
        };
        assert_eq!(denied_operation, "SetRetention", "{corrupt}");
    }
}
#[test]
fn typed_finish_requires_the_exact_source_write_not_an_independent_input_read() {
    use super::{super::schema::*, declaration::*};
    let result = SetRetention::reference::<DocumentRetentionSchema<120>>()
        .body()
        .retire_owned_selections_on_input_change(
            DocumentRetentionField::reference(),
            "retention_days",
            OwnedRows::reference(),
            CurrentSelection::reference(),
            std::num::NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
        .assign(DocumentRetentionField::reference(), 8)
        .unwrap()
        .finish();
    assert!(matches!(
        result,
        Err(ApplicationOperationBodyAuthoringDenial::InvalidSelectionRetirement)
    ));
}
/// Installation needs declared Delete and Unlink authority, not a supplier
/// forecast of candidate cardinality (owned-selection-retirement.md, "the
/// native owner grows actual effect storage fallibly"). Missing authority
/// refuses at declaration above; exceeding the bound refuses at execution
/// before any prefix (`evidence::bound_overflow_and_overlapping_targets_never_apply_a_prefix`).
/// So the program that once carried no forecast installs and bootstraps.
#[test]
fn installation_admits_the_retirement_bound_without_candidate_forecasts() {
    // The real fixture bootstrap binds the operator and seeds both holders.
    let app = super::super::install::<125>();
    assert_eq!(super::super::read_retention(&app), 7);
}
