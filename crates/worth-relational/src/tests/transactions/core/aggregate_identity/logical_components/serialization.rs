//! Required content authority preserves explicit empty preceding tail slots.
use super::checkpoint::configured_runtime;
use super::fixtures::*;
use crate::durability::data::DurableBranchRootSchemaImage;
use serde::{de::IgnoredAny, Serialize};

/// An independent serializer for the required seven-field authority contract.
#[derive(Serialize)]
struct RequiredAuthority<'a> {
    primary_schema_id: &'a Option<SchemaId>,
    primary_schema_version_id: &'a Option<SchemaVersionId>,
    entity_kinds: &'a Vec<SchemaAuthorityKindSnapshot>,
    relation_kinds: &'a Vec<SchemaAuthorityRelationSnapshot>,
    derived_relations: &'a Vec<DerivedRelationDefinitionParts>,
    projected_member_domains: &'a Vec<ProjectedMemberDomainParts>,
    content_formats: &'a crate::content::RelationalContentSchemaSnapshot,
}

#[test]
fn required_content_authority_keeps_exact_shape_and_positional_empty_tails() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for domain_only in [false, true] {
        let mut registry = RelationIntegritySchemaFixture::default().build_registry();
        if domain_only {
            let (schema, version) = {
                let carrier = registry.entity_registration(KindId(1)).unwrap();
                (carrier.schema_id.clone(), carrier.schema_version_id)
            };
            registry = registry
                .register_projected_member_domains(vec![ProjectedMemberDomainParts {
                    identity: ProjectedMemberDomainIdentity {
                        schema,
                        version,
                        name: "Uses".into(),
                    },
                    carrier: KindId(1),
                    cases: CASES.iter().map(|case| (*case).to_owned()).collect(),
                }])
                .unwrap();
        }
        let authority = registry.authority_snapshot();
        assert!(authority.derived_relations.is_empty());
        let compact = rmp_serde::to_vec(&authority).unwrap();
        let fields: Vec<IgnoredAny> = rmp_serde::from_slice(&compact).unwrap();
        assert_eq!(fields.len(), 7);
        let (_, _, _, _, derived, domains, content): (
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            Vec<DerivedRelationDefinitionParts>,
            Vec<ProjectedMemberDomainParts>,
            crate::content::RelationalContentSchemaSnapshot,
        ) = rmp_serde::from_slice(&compact).unwrap();
        assert!(derived.is_empty());
        assert_eq!(domains, authority.projected_member_domains);
        assert_eq!(domains.is_empty(), !domain_only);
        assert_eq!(
            content,
            crate::content::RelationalContentSchemaSnapshot::default()
        );
        let expected = RequiredAuthority {
            primary_schema_id: &authority.primary_schema_id,
            primary_schema_version_id: &authority.primary_schema_version_id,
            entity_kinds: &authority.entity_kinds,
            relation_kinds: &authority.relation_kinds,
            derived_relations: &authority.derived_relations,
            projected_member_domains: &authority.projected_member_domains,
            content_formats: &authority.content_formats,
        };
        assert_eq!(compact, rmp_serde::to_vec(&expected).unwrap());
        assert_eq!(
            serde_json::to_vec(&authority).unwrap(),
            serde_json::to_vec(&expected).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec_named(&authority).unwrap(),
            rmp_serde::to_vec_named(&expected).unwrap()
        );
        for bytes in [compact, rmp_serde::to_vec_named(&authority).unwrap()] {
            assert_eq!(
                rmp_serde::from_slice::<SchemaAuthoritySnapshot>(&bytes).unwrap(),
                authority
            );
        }
        assert_eq!(
            serde_json::from_slice::<SchemaAuthoritySnapshot>(
                &serde_json::to_vec(&authority).unwrap()
            )
            .unwrap(),
            authority
        );

        let runtime = configured_runtime(registry);
        commit(
            execution,
            &runtime,
            WorkerIntentBatch::new("serialized-authority").push(entity(1, 0)),
        );
        let checkpoint = runtime
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        let image = &checkpoint.branch_root_schema_images[0];
        let compact = rmp_serde::to_vec(image).unwrap();
        let fields: Vec<IgnoredAny> = rmp_serde::from_slice(&compact).unwrap();
        assert_eq!(fields.len(), 10);
        let (_, _, _, _, _, _, _, derived, domains, content): (
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            IgnoredAny,
            Vec<DerivedRelationDefinitionParts>,
            Vec<ProjectedMemberDomainParts>,
            crate::content::RelationalContentSchemaSnapshot,
        ) = rmp_serde::from_slice(&compact).unwrap();
        assert!(derived.is_empty());
        assert_eq!(domains, authority.projected_member_domains);
        assert_eq!(
            content,
            crate::content::RelationalContentSchemaSnapshot::default()
        );
        for bytes in [compact, rmp_serde::to_vec_named(image).unwrap()] {
            let decoded: DurableBranchRootSchemaImage = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(&decoded, image);
            assert_eq!(
                decoded.readmit_registry().unwrap().0.authority_snapshot(),
                authority
            );
        }
        let decoded: DurableBranchRootSchemaImage =
            serde_json::from_slice(&serde_json::to_vec(image).unwrap()).unwrap();
        assert_eq!(
            decoded.readmit_registry().unwrap().0.authority_snapshot(),
            authority
        );
    }
}
