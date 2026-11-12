//! Live-owner relation contracts and the kinds covered by their field demands.
use super::*;
#[test]
fn a_live_member_owner_requires_its_exact_relation_demand_and_cardinality() {
    use worth_query_declaration::facade::application_aftermath::{
        PortablePreImageLocus, PortablePreImageRelationLocus,
    };
    use worth_query_declaration::facade::application_schema::{
        ApplicationRelationCardinality, ApplicationRelationIntegrity,
    };
    let families = (
        RetiredMembersBinding::descriptor(),
        CreatedMembersBinding::descriptor(),
    );
    // A live owner's members are role families; an exactly-one role is refused.
    let exact = (RetiredBinding::descriptor(), CreatedBinding::descriptor());
    let queries = [disclosure::HistoryBinding::descriptor()];
    let field = worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
        InstallationFields,
        FixtureEntity,
        FixtureIdentityAspect,
        u64,
    >::at("PrincipalIdentityField");
    let key = ApplicationCorrectionOutput::<CreatedBinding, Created>::field(field)
        .declaration()
        .unwrap()
        .projected_value_member();
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        ..
    } = &key
    else {
        unreachable!();
    };
    let owner = PortablePreImageRelationLocus::from_untrusted_fields(
        "MemberOf".into(),
        entity.clone(),
        "Owner".into(),
    );
    for axis in [
        "valid",
        "owner-field",
        "foreign-field",
        "missing-owner-demand-with-field",
        "missing-demand",
        "wrong-direction",
        "missing-relation",
        "optional-owner",
        "many-owners",
        "exact-roles",
    ] {
        let (source, inverse) = if axis == "exact-roles" {
            &exact
        } else {
            &families
        };
        let inventory = BTreeMap::from([
            (source.identity().to_owned(), source),
            (inverse.identity().to_owned(), inverse),
        ]);
        let roles = if axis == "exact-roles" {
            ("retired", "created")
        } else {
            ("retired.", "created.")
        };
        let declared_owner = if axis == "wrong-direction" {
            PortablePreImageRelationLocus::from_untrusted_fields(
                "MemberOf".into(),
                "Owner".into(),
                entity.clone(),
            )
        } else {
            owner.clone()
        };
        let authored =
            DeclaredAuthoredCorrection::from_untrusted_parts(DeclaredAuthoredCorrectionParts {
                family: identity("authored-fixture.v1"),
                logical_key: Box::new(key.clone()),
                binding_unit: None,
                source_role: roles.0.into(),
                inverse_role: roles.1.into(),
                source_posture: Posture::Retire,
            })
            .with_live_member_owner(declared_owner.clone());
        let mut association = parts(source);
        association.inverse = inverse.description().clone();
        association.inverse_handler = inverse.handler().identity().to_owned();
        association.authored_identity = Some(authored);
        let mut loci = vec![PortablePreImageLocus::from_untrusted_fields(
            entity.clone(),
            aspect.clone(),
            field.clone(),
        )];
        if matches!(
            axis,
            "owner-field" | "foreign-field" | "missing-owner-demand-with-field"
        ) {
            loci.push(PortablePreImageLocus::from_untrusted_fields(
                if axis == "foreign-field" {
                    "Unrelated"
                } else {
                    "Owner"
                }
                .into(),
                "OwnerFacts".into(),
                "OwnerValue".into(),
            ));
        }
        let demand = PortablePreImageDemand::from_untrusted_fields(loci, 1024)
            .with_untrusted_relations(
                if matches!(axis, "missing-demand" | "missing-owner-demand-with-field") {
                    vec![]
                } else {
                    vec![declared_owner.clone()]
                },
            );
        let contract = PortableApplicationAftermathContract::from_untrusted_fields(
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    inverse.operation_name().to_owned(),
                    DeclaredLoweringCorrespondenceRef::new("fixture.authored.inverse.v1"),
                    DeclaredAftermathPostcondition::ExactPriorTruth,
                    demand,
                ),
            )),
            None,
        )
        .with_untrusted_correction_binding(
            DeclaredCorrectionBinding::from_untrusted_parts(association),
        );
        let mut actual = vec![
            key.clone(),
            ApplicationSchemaMember::OperationAftermath {
                operation: source.operation_name().to_owned(),
                contract,
            },
            ApplicationSchemaMember::OperationProgram {
                operation: source.operation_name().to_owned(),
                target: Target::Delete {
                    entity: entity.clone(),
                },
            },
            ApplicationSchemaMember::OperationProgram {
                operation: inverse.operation_name().to_owned(),
                target: Target::Create {
                    entity: entity.clone(),
                },
            },
        ];
        if axis != "missing-relation" {
            let mut integrity =
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
            integrity.cardinality = ApplicationRelationCardinality::new(
                Some(if axis == "optional-owner" { 0 } else { 1 }),
                Some(if axis == "many-owners" { 2 } else { 1 }),
                None,
                None,
                None,
                None,
            );
            actual.push(ApplicationSchemaMember::Relation {
                relation: "MemberOf".into(),
                from: declared_owner.from().to_owned(),
                to: declared_owner.to().to_owned(),
                integrity: integrity.into(),
            });
        }
        let result = resolve_correction_binding(&actual, source, &inventory, &queries);
        if matches!(axis, "valid" | "owner-field") {
            assert!(result.unwrap().is_some());
        } else if matches!(axis, "foreign-field" | "missing-owner-demand-with-field") {
            let denial = result.unwrap_err();
            assert_eq!(denial.kind(),crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::PreImageDemandNotCoveredByAuthoredAnchors,"{axis}");
            assert_eq!(
                denial.operation(),
                if axis == "foreign-field" {
                    "Unrelated"
                } else {
                    "Owner"
                }
            );
        } else {
            assert_eq!(result.unwrap_err().kind(), crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied, "{axis}");
        }
    }
}
