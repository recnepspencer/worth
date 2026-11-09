//! Contract compilation only: inert capture records never represent signing authority.
#[path = "tests/records.rs"]
mod storage_model;
use super::*;
use storage_model::*;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::{application_query::*, application_schema::*};
struct Schema;

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.lifecycle-path-contract";
    const NAME: &'static str = "LifecyclePathContract";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_declaration::facade::application_schema::fragment::declare(Self)
    }
}
struct Fixture {
    binding: ApplicationSchemaBindingIdentity,
    catalog: Catalog,
    source: ApplicationOperationBodyPathFieldSource,
    touches: Contract,
}
impl Fixture {
    fn new() -> Self {
        let schema = Schema::declaration().unwrap();
        let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
            1,
            1,
            CanonicalDigestId::new([1; 32]),
            CanonicalDigestId::new([2; 32]),
        );
        let (_, work) =
            crate::application_schema::derive_installed_schema_identity(schema.erased().identity())
                .unwrap();
        let native =
            crate::application_schema::compile_portable_native_contract_records(schema.erased())
                .unwrap();
        let catalog =
            crate::application_schema::compile_native_contract_catalog(&binding, &native, work)
                .unwrap();
        let field = Locus::new("Parent", Revision::at("Revision").aspect(), "Revision");
        let source = ApplicationOperationBodyPathFieldSource {
            path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationQueryRootPathParts {
                    start_entity: "Row".into(),
                    terminal_entity: "Parent".into(),
                    guards: vec![],
                    steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                        "ParentRows".into(),
                        "Parent".into(),
                        "Row".into(),
                        RelationDirection::Reverse,
                     worth_query_declaration::facade::application_query::ApplicationQueryRootPathTraversal::OneHop,)],
                },
            ),
            field,
            presence: ApplicationFieldPresence::Required,
        };
        let touches = Contract::Declared {
            graph_roles: vec![],
            scopes: vec![Touch::WriteField(WorthQueryOperationFieldTouchScope::new(
                binding.clone(),
                "Parent".into(),
                catalog
                    .aspect("Parent", Revision::at("Revision").aspect())
                    .unwrap(),
                CanonicalFieldPath::single(FieldKey::new("Revision").unwrap()),
            ))],
        };
        Self {
            binding,
            catalog,
            source,
            touches,
        }
    }
    fn derive(
        &self,
        role: Role,
        maximum_subjects: Option<std::num::NonZeroUsize>,
    ) -> SigningLifecycleClosure {
        let creation = ApplicationOperationBodyCreation::from_untrusted(
            "Evidence".into(),
            "evidence".into(),
            vec![Field::CapturedPathField {
                field: Locus::new("Evidence", "Facts", "SignedRevision"),
                binding: U64ApplicationValueBinding::IDENTITY,
                scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
                source: self.source.clone(),
                role,
                target_presence: ApplicationFieldPresence::Required,
            }],
        )
        .with_untrusted_signing(Some(ApplicationOperationBodySigningSpec::from_untrusted(
            "verify".into(),
            Locus::new("Evidence", "Facts", "Actor"),
            Locus::new("Evidence", "Facts", "Provenance"),
        )))
        .with_untrusted_signing_lifecycle(Some(Spec {
            history_relation: "History".into(),
            current_relation: "Current".into(),
            source_entity: "Row".into(),
            consequence_relation: "Consequences".into(),
            consequence_field: Locus::new("Consequence", Record::at("Record").aspect(), "Record"),
            maximum_evidence: 2,
        }))
        .with_untrusted_subject_impact_bound(maximum_subjects);
        let members = vec![Member::OperationBody {
            operation: "Sign".into(),
            body: ApplicationOperationBodyProgram::from_untrusted_in_scope(
                "Row".into(),
                vec![],
                Some(creation),
                vec![],
            ),
        }];
        let writer = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Row".into(),
            vec![],
            None,
            vec![],
        )
        .with_untrusted_path_value_assignments(vec![
            ApplicationOperationBodyPathValueAssignment {
                target: self.source.clone(),
                binding: U64ApplicationValueBinding::IDENTITY,
                value: "next".into(),
            },
        ]);
        SigningLifecycleClosure::derive(
            &self.binding,
            &self.catalog,
            &members,
            &self.touches,
            Some(&writer),
        )
        .unwrap()
    }
}
#[test]
fn path_only_subject_writes_compile_lifecycle_consequences_but_audit_context_does_not() {
    let f = Fixture::new();
    let subject = f.derive(Role::Subject, std::num::NonZeroUsize::new(3));
    assert_eq!(subject.rules.len(), 1);
    assert_eq!(
        subject.rules[0].subjects(),
        std::slice::from_ref(&f.source.field)
    );
    assert_eq!(subject.rules[0].maximum_subjects(), Some(3));
    assert_eq!(
        f.derive(Role::Subject, None).rules[0].maximum_subjects(),
        None
    );
    assert!(subject
        .publication_touches
        .scopes()
        .iter()
        .any(|touch| matches!(touch,Touch::CreateEntity(scope) if scope.entity()=="Consequence")));
    assert!(subject
        .publication_touches
        .scopes()
        .iter()
        .any(|touch| matches!(touch,Touch::WriteField(scope) if scope.entity()=="Consequence")));
    assert!(subject.publication_touches.scopes().iter().any(
        |touch| matches!(touch,Touch::LinkRelation(scope) if scope.relation()=="Consequences")
    ));
    assert!(subject
        .publication_touches
        .scopes()
        .iter()
        .any(|touch| matches!(touch,Touch::UnlinkRelation(scope) if scope.relation()=="Current")));
    let audit = f.derive(Role::AuditContext, None);
    assert!(audit.rules.is_empty());
    assert_eq!(audit.publication_touches, f.touches);
}

impl worth_query_declaration::facade::application_schema::fragment::Fragment for Schema {
    const NAME: &'static str = "lifecycle";
    type Bindings = worth_query_declaration::facade::application_schema::fragment::NoBindings;
    type Members = (LifecycleRecords,);
}
