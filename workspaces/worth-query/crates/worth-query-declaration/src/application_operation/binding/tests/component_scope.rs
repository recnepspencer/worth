//! Actual mutation scope metadata closes component paths at fresh schema admission.
use super::*;
use crate::application_query::{
    ApplicationQueryRootPath, ApplicationQueryRootPathGuard, ApplicationQueryRootPathMeaning,
    WorthQueryPortableApplicationQueryRootPathGuardParts,
    WorthQueryPortableApplicationQueryRootPathParts,
};
use crate::application_schema::*;
struct Accounts;
impl OperationReads<RenameOperation> for Accounts {}
struct Coupled;
fn typed_scope() -> ApplicationOperationComponentReadMeaning {
    let path = || {
        ApplicationQueryRootPath::from(Account::reference()).where_equal(
            AccountIdField::reference(),
            ApplicationEncodedScalarValue::try_new(7).unwrap(),
        )
    };
    ApplicationOperationComponentReadDefinition::new(
        RenameOperation::reference(),
        ApplicationComponentSetRef::<MutationSchema, Accounts, Account>::new(
            "Accounts",
            Account::reference(),
        ),
        path(),
    )
    .also_path(path())
    .into_meaning()
}
fn schema(
    scope: ApplicationOperationComponentReadMeaning,
) -> WorthQueryPortableApplicationSchemaRecord {
    let mut parts = WorthQueryPortableApplicationSchemaRecord::project(
        MutationSchema::declaration().unwrap().erased(),
    )
    .into_parts();
    let relation = ApplicationRelationRef::<MutationSchema, Coupled, Account, Account>::from_schema_identifiers("Coupled", "Account", "Account", ApplicationRelationIntegrity::same_context_unbounded_retain_dangling());
    parts.members.extend([
        ApplicationSchemaMember::Relation {
            relation: "Coupled".into(),
            from: "Account".into(),
            to: "Account".into(),
            integrity: relation.integrity().into(),
        },
        ApplicationSchemaMember::ComponentSet {
            definition: ApplicationComponentSetDefinition::new(ApplicationComponentSetRef::<
                MutationSchema,
                Accounts,
                Account,
            >::new(
                "Accounts", Account::reference()
            ))
            .coupled_by(relation)
            .into_meaning(),
        },
        ApplicationSchemaMember::OperationComponentRead { definition: scope },
    ]);
    parts.members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
}
fn parts(
    scope: &ApplicationOperationComponentReadMeaning,
) -> WorthQueryPortableOperationComponentReadParts {
    WorthQueryPortableOperationComponentReadParts {
        operation: scope.operation().into(),
        set: scope.set().into(),
        root: scope.root().into(),
        member: scope.member().into(),
        paths: scope.paths().to_vec(),
    }
}
#[test]
fn authored_component_scope_deduplicates_paths_and_binds_actual_mutation_root() {
    let scope = typed_scope();
    assert_eq!(scope.paths().len(), 1);
    let record = schema(scope.clone());
    let accepted = validate_portable_application_schema_freshly(record.clone()).unwrap();
    assert_eq!(
        validate_portable_application_schema_freshly(record)
            .unwrap()
            .identity(),
        accepted.identity()
    );
    let mut changed = parts(&scope);
    changed.paths[0] = ApplicationQueryRootPath::from(Account::reference()).into_meaning();
    let changed = validate_portable_application_schema_freshly(schema(
        ApplicationOperationComponentReadMeaning::from_untrusted_parts(changed),
    ))
    .unwrap();
    assert_ne!(
        accepted.identity(),
        changed.identity(),
        "visibility changes schema meaning, independently of component identity"
    );
}
#[test]
fn untrusted_component_scope_refuses_forged_roots_guards_and_noncanonical_path_unions() {
    let scope = typed_scope();
    let path = &scope.paths()[0];
    for index in 0..5 {
        let mut changed = parts(&scope);
        match index {
            0 => changed.operation = "MissingOperation".into(),
            1 => {
                changed.root = "ExternalMapping".into();
                changed.paths = vec![ApplicationQueryRootPath::from(ExternalMapping::reference())
                    .forward(MappingTarget::reference())
                    .into_meaning()];
            }
            2 => changed.paths.push(path.clone()),
            3 => changed.paths.clear(),
            4 => {
                let guard = &path.guards()[0];
                changed.paths[0] = ApplicationQueryRootPathMeaning::from_untrusted_parts(WorthQueryPortableApplicationQueryRootPathParts {
                    start_entity: "Account".into(), terminal_entity: "Account".into(), steps: vec![], guards: vec![ApplicationQueryRootPathGuard::from_untrusted_parts(WorthQueryPortableApplicationQueryRootPathGuardParts {
                        after_step: 0, entity: "Account".into(), aspect: guard.aspect().into(), field: "MissingField".into(), scalar_family: guard.scalar_family(), value_type: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(guard.value_type().into()), expected: guard.expected().clone(),
                    })],
                });
            }
            _ => unreachable!(),
        }
        assert_eq!(
            validate_portable_application_schema_freshly(schema(
                ApplicationOperationComponentReadMeaning::from_untrusted_parts(changed)
            ))
            .unwrap_err(),
            ApplicationSchemaDeclarationDenial::InvalidOperationComponentRead,
            "case {index}"
        );
    }
}
