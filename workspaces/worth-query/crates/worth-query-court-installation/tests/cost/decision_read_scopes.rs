//! Independent width/reference axes measure portable metadata, separately from insertion work.
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;

fn package(width: usize, references: usize) -> WorthQueryValidatedPortableDomainPackage {
    let mut members = (0..width)
        .map(|index| ApplicationSchemaMember::Entity {
            entity: format!("Entity{index}"),
        })
        .collect::<Vec<_>>();
    let definition = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
        (0..width)
            .map(|index| ApplicationOperationDecisionReadTarget::Entity {
                entity: format!("Entity{index}"),
            })
            .collect(),
        Vec::new(),
    )
    .unwrap();
    for index in 0..references {
        let operation = format!("Read{index}");
        members.push(ApplicationSchemaMember::Operation { operation: operation.clone(), input_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared("worth.rust.unit") });
        members.push(ApplicationSchemaMember::OperationDecisionReadScope {
            operation,
            scope: definition.identity().to_owned(),
        });
    }
    members.push(ApplicationSchemaMember::DecisionReadScope { definition });
    members.sort();
    let schema = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
            WorthQueryPortableApplicationSchemaParts {
                owner: "worth.tests.shared".into(),
                name: "SharedReads".into(),
                major: 1,
                minor: 0,
                members,
                contributions: Vec::new(),
            },
        ),
    )
    .unwrap();
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.tests.shared",
        1,
        0,
    ))
    .application_schema_erased(schema)
    .validate()
    .unwrap()
}

fn measure(width: usize, references: usize) -> [u64; 3] {
    let package = package(width, references);
    let bytes = package.canonical_work().canonical_encoded_bytes();
    let exported = package.export_typed_records().unwrap();
    let index = crate::installation::aspect_native_identity::model::index_from(package);
    println!("shared scope width={width} references={references} package_canonical_bytes={bytes} export_logical_bytes={} index_construction_canonical_bytes={}",
        exported.manifest().logical_export_bytes(), index.installation_canonical_work().canonical_encoded_bytes());
    [
        bytes.try_into().unwrap(),
        exported.manifest().logical_export_bytes(),
        index
            .installation_canonical_work()
            .canonical_encoded_bytes()
            .try_into()
            .unwrap(),
    ]
}

#[test]
fn common_read_metadata_grows_with_scope_width_plus_operation_references() {
    let mut bytes = [[[0; 3]; 3]; 3];
    for (row, width) in [1, 10, 100].into_iter().enumerate() {
        for (column, references) in [1, 10, 100].into_iter().enumerate() {
            bytes[row][column] = measure(width, references);
        }
    }
    for (axis, name) in [
        "package canonical",
        "export logical",
        "index construction canonical",
    ]
    .into_iter()
    .enumerate()
    {
        let narrow_delta = bytes[0][2][axis] - bytes[0][1][axis];
        let wide_delta = bytes[2][2][axis] - bytes[2][1][axis];
        assert!(
            wide_delta < narrow_delta * 2,
            "{name}: adding references must not copy their scope width"
        );
        let few_delta = bytes[2][0][axis] - bytes[1][0][axis];
        let many_delta = bytes[2][2][axis] - bytes[1][2][axis];
        assert!(
            many_delta < few_delta * 2,
            "{name}: scope content must not be embedded per operation"
        );
    }
}
