//! Flat record authoring and hostile scalar descriptors meet the same compiler.
use super::*;
use worth_query_declaration::facade::application_schema::*;
mod installed_read_scope;
mod model;
use model::*;

fn scalar(extra: bool, optional: bool) -> ErasedApplicationSchemaDeclaration {
    let source = FlatSchema::declaration().unwrap();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(source.erased()).into_parts();
    let field = value();
    parts
        .members
        .push(ApplicationSchemaMember::ScalarAspectField {
            entity: field.entity().into(),
            aspect: field.aspect().into(),
            field: field.field().into(),
        });
    if extra {
        let sibling = parts
            .members
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::Field { .. } => Some(member.clone()),
                _ => None,
            })
            .unwrap();
        let mut sibling = sibling;
        if let ApplicationSchemaMember::Field { field, .. } = &mut sibling {
            *field = "sibling".into();
        }
        parts.members.push(sibling);
    }
    if optional {
        for member in &mut parts.members {
            if let ApplicationSchemaMember::Field { presence, .. } = member {
                *presence = ApplicationFieldPresence::Optional;
            }
        }
    }
    parts.members.sort();
    parts.members.dedup();
    validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap()
}

#[test]
fn scalar_compilation_checks_the_exact_required_sole_field() {
    let flat = FlatSchema::declaration().unwrap();
    let structures = compile_portable_native_contract_records(flat.erased()).unwrap();
    assert!(matches!(
        structures[0].contract().shape(),
        worth_foundational::facade::AspectShape::Struct(_)
    ));
    let scalar = scalar(false, false);
    let records = compile_portable_native_contract_records(&scalar).unwrap();
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0].contract().shape(),
        worth_foundational::facade::AspectShape::Scalar(ScalarAspectType::UInt64)
    ));
    assert!(
        matches!(records[0].binding(), AspectBinding::EntityField { field } if field.as_str() == "value")
    );
    let scalar_meaning = ApplicationAspectValueMeaning::from_members(scalar.members())
        .unwrap()
        .remove(0);
    let struct_meaning = ApplicationAspectValueMeaning::from_members(flat.erased().members())
        .unwrap()
        .remove(0);
    assert_eq!(
        scalar_meaning.compare_predecessor(&struct_meaning),
        Err(ApplicationAspectMeaningPredecessorDenial::MeaningChangedWithoutRevision)
    );
    for invalid in [self::scalar(true, false), self::scalar(false, true)] {
        assert_eq!(
            compile_portable_native_contract_records(&invalid)
                .unwrap_err()
                .kind(),
            DenialKind::InvalidAspectShape
        );
    }
}
