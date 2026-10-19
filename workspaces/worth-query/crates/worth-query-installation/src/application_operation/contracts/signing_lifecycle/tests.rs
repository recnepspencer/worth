//! Contract compilation only: inert capture records never represent signing authority.
use super::*;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::{
    facade::{application_query::*, application_schema::*},
    worth_query_aspect, worth_query_entity, worth_query_field,
};
struct Schema;
worth_query_entity!(Parent for Schema);
worth_query_entity!(Consequence for Schema);
worth_query_aspect!(ParentFacts for Schema, Parent; identity = AspectIdentity(0x9175_E001), revision = AspectContractRevision(1),);
worth_query_aspect!(ConsequenceFacts for Schema, Consequence; identity = AspectIdentity(0x9175_E002), revision = AspectContractRevision(1),);
worth_query_field!(Revision for Schema, Parent, ParentFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(Record for Schema, Consequence, ConsequenceFacts: ApplicationSigningSupersession => ApplicationSigningSupersessionBinding, read_only, equality);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.lifecycle-path-contract";
    const NAME: &'static str = "LifecyclePathContract";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(Parent::reference())
            .aspect(Parent::reference(), ParentFacts::reference())
            .field(Parent::reference(), Revision::reference())
            .entity(Consequence::reference())
            .aspect(Consequence::reference(), ConsequenceFacts::reference())
            .field(Consequence::reference(), Record::reference())
            .build()
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
        let field = Locus::new("Parent", "ParentFacts", "Revision");
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
                        ApplicationQueryRootPathDirection::Reverse,
                    )],
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
                catalog.aspect("Parent", "ParentFacts").unwrap(),
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
    fn derive(&self, role: Role) -> SigningLifecycleClosure {
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
            consequence_field: Locus::new("Consequence", "ConsequenceFacts", "Record"),
            maximum_evidence: 2,
        }))
        .with_untrusted_subject_impact_bound(std::num::NonZeroUsize::new(3));
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
            "WriteParent",
            &self.touches,
            99,
            Some(&writer),
        )
        .unwrap()
    }
}
#[test]
fn path_only_subject_writes_compile_lifecycle_consequences_but_audit_context_does_not() {
    let f = Fixture::new();
    let subject = f.derive(Role::Subject);
    assert_eq!(subject.rules.len(), 1);
    assert_eq!(subject.rules[0].subjects(), [f.source.field.clone()]);
    assert_eq!(subject.rules[0].maximum_subjects(), 3);
    let ceiling = subject
        .ceiling
        .expect("actual consequence capacity")
        .cardinality();
    assert_eq!(
        (
            ceiling.maximum_creates(),
            ceiling.maximum_links(),
            ceiling.maximum_unlinks(),
            ceiling.maximum_writes()
        ),
        (6, 6, 6, 6)
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
    let audit = f.derive(Role::AuditContext);
    assert!(audit.rules.is_empty());
    assert!(audit.ceiling.is_none());
    assert_eq!(audit.publication_touches, f.touches);
}
