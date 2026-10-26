//! Endpoint direction and declared membership survive fresh native readmission.
use super::*;
use worth_query_host::facade::declaration::application_schema::*;

#[test]
fn portable_creation_links_refuse_wrong_direction_endpoint_and_membership() {
    let schema = program::schema::<30>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable.clone()).unwrap(),
        *schema.erased()
    );
    for defect in ["direction", "endpoint", "key", "membership"] {
        let mut parts = portable.clone().into_parts();
        for member in &mut parts.members {
            if let ApplicationSchemaMember::OperationBody { operation, body } = member {
                if operation != "CreateDocument" {
                    continue;
                }
                let creation = body.creation().unwrap();
                let mut links = creation.links().to_vec();
                match defect {
                    "direction" => {
                        links[0].direction = ApplicationOperationBodyCreationLinkDirection::ToScope
                    }
                    "endpoint" => links[0].to = "Principal".to_owned(),
                    "key" => links[0].key = " ".to_owned(),
                    "membership" => {}
                    _ => unreachable!(),
                }
                let creation = creation.clone().with_untrusted_links(links);
                *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                    body.scope_entity().unwrap().to_owned(),
                    body.scope_fields().to_vec(),
                    Some(creation),
                    body.requirements().to_vec(),
                );
            }
            if defect == "membership"
                && let ApplicationSchemaMember::OperationProgram {
                    operation,
                    target: ApplicationOperationProgramTarget::Link { relation, .. },
                } = member
                && operation == "CreateDocument"
                && relation == "TenantDocuments"
            {
                *relation = "DocumentTenant".to_owned();
            }
        }
        parts.members.sort();
        assert!(
            validate_portable_application_schema_freshly(
                WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
            )
            .is_err(),
            "{defect}"
        );
    }
}
