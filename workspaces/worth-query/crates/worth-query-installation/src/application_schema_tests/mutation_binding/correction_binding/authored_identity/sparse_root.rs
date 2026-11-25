//! A sparse preserved root installs beside live-owner member families only.
use super::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_aftermath::{
    PortablePreImageFieldOutput, PortablePreImageLocus, PortablePreImageRelationLocus,
    PreImageOutputFieldCapture,
};
use worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput;
use worth_query_declaration::facade::application_schema::{
    fragment::Entity, ApplicationRelationCardinality, ApplicationRelationIntegrity,
};

struct OwnerEntity;
impl Entity for OwnerEntity {
    const NAME: &'static str = "Owner";
    type Aspects = ();
    type Relations = ();
}
macro_rules! revise {
    ($Outputs:ident, $Root:ident, $Retired:ident, $Created:ident, $RootEntity:ty, $Binding:ident, $id:literal) => {
        struct $Outputs;
        struct $Root;
        struct $Retired;
        struct $Created;
        impl WorthQueryApplicationOutputRole for $Root {
            type Schema = MutationSchema;
            type Contract = $Outputs;
            type Entity = $RootEntity;
            type Action = WorthQueryPreserveOutput;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = "root";
        }
        impl WorthQueryApplicationOutputRoleFamily for $Retired {
            type Schema = MutationSchema;
            type Contract = $Outputs;
            type Entity = FixtureEntity;
            const PREFIX: &'static str = "retired.";
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::one(Posture::Retire);
            const MINIMUM: usize = 0;
        }
        impl WorthQueryApplicationOutputRoleFamily for $Created {
            type Schema = MutationSchema;
            type Contract = $Outputs;
            type Entity = FixtureEntity;
            const PREFIX: &'static str = "created.";
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::one(Posture::Create);
            const MINIMUM: usize = 0;
        }
        impl ApplicationMutationOutputContract<MutationSchema> for $Outputs {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[$Root::DESCRIPTOR];
            const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
                <$Retired as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
                <$Created as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
            ];
        }
        mutation_binding!(
            $Binding,
            $id,
            TestOperation<MutationSchema>,
            TestInput,
            TestInputBinding,
            MutationDecision,
            MutationDenialBinding,
            "fixture.sparse-root.revise-handler.v1",
            FixturePrincipalIdentityField::at("PrincipalIdentityField"),
            test_principal_binding::<MutationSchema>(),
            $Outputs
        );
    };
}
revise!(
    ReviseOutputs,
    Root,
    RetiredRoster,
    CreatedRoster,
    OwnerEntity,
    ReviseBinding,
    "fixture.sparse-root.revise.v1"
);
revise!(
    SameOutputs,
    SameRoot,
    SameRetired,
    SameCreated,
    FixtureEntity,
    SameBinding,
    "fixture.sparse-root.same.v1"
);

fn locus(entity: &str, aspect: &str, field: &str) -> PortablePreImageLocus {
    PortablePreImageLocus::from_untrusted_fields(
        ARCHIVE_DECODE,
        entity.into(),
        aspect.into(),
        field.into(),
    )
}

/// One self-inverse revision of a preserved owner: it retires and creates
/// keyed members through two live-owner families while its own fields are
/// captured sparsely. Each refusal axis breaks exactly one rule.
#[test]
fn a_sparse_root_installs_only_beside_live_owner_member_families() {
    use crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind as Kind;
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
    let revise = ReviseBinding::descriptor();
    let same = SameBinding::descriptor();
    for (axis, denial) in [
        ("valid", None),
        ("anchored", Some(Kind::AftermathInstallationDenied)),
        // An owner on a third entity leaves the root locus uncovered by anchors.
        (
            "foreign-owner-locus",
            Some(Kind::PreImageDemandNotCoveredByAuthoredAnchors),
        ),
        (
            "unrelated-locus",
            Some(Kind::PreImageDemandNotCoveredByAuthoredAnchors),
        ),
        ("family-is-root", Some(Kind::AftermathInstallationDenied)),
        ("no-root-locus", Some(Kind::AftermathInstallationDenied)),
        (
            "missing-owner-demand",
            Some(Kind::PreImageDemandNotCoveredByAuthoredAnchors),
        ),
    ] {
        let binding = if axis == "family-is-root" {
            &same
        } else {
            &revise
        };
        let root_entity = if axis == "family-is-root" {
            entity.as_str()
        } else {
            "Owner"
        };
        let owner_entity = if axis == "foreign-owner-locus" {
            "Elsewhere"
        } else {
            "Owner"
        };
        let owner = PortablePreImageRelationLocus::from_untrusted_fields(
            ARCHIVE_DECODE,
            "MemberOf".into(),
            entity.clone(),
            owner_entity.into(),
        );
        let family = |source: &str, inverse: &str, posture| {
            let parts = DeclaredAuthoredCorrectionParts {
                family: identity("sparse-root-member.v1"),
                logical_key: Box::new(key.clone()),
                binding_unit: None,
                source_role: source.into(),
                inverse_role: inverse.into(),
                source_posture: posture,
            };
            let declared = DeclaredAuthoredCorrection::from_untrusted_parts(ARCHIVE_DECODE, parts);
            if axis == "anchored" {
                declared
            } else {
                declared.with_live_member_owner(owner.clone())
            }
        };
        let authored = family("retired.", "created.", Posture::Retire)
            .with_co_anchor(family("created.", "retired.", Posture::Create), false);
        let mut association = parts(binding);
        association.authored_identity = Some(authored);
        let mut loci = Vec::new();
        if axis != "no-root-locus" {
            loci.push(if axis == "family-is-root" {
                locus(entity, "OtherAspect", "OtherField")
            } else {
                locus("Owner", "OwnerFacts", "OwnerValue")
            });
        }
        if axis != "anchored" {
            loci.push(locus(entity, aspect, field));
        }
        match axis {
            "foreign-owner-locus" => loci.push(locus("Elsewhere", "Facts", "Value")),
            "unrelated-locus" => loci.push(locus("Unrelated", "Facts", "Value")),
            _ => (),
        }
        let demand = PortablePreImageDemand::from_untrusted_fields(ARCHIVE_DECODE, loci, 1024)
            .with_untrusted_output_fields(PortablePreImageFieldOutput::from_untrusted_fields(
                ARCHIVE_DECODE,
                "root".into(),
                root_entity.into(),
                PreImageOutputFieldCapture::ChangedPreserved,
            ))
            .with_untrusted_relations(if axis == "missing-owner-demand" {
                vec![]
            } else {
                vec![owner.clone()]
            });
        let contract = PortableApplicationAftermathContract::from_untrusted_fields(
            ARCHIVE_DECODE,
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    ARCHIVE_DECODE,
                    binding.operation_name().to_owned(),
                    DeclaredLoweringCorrespondenceRef::new("fixture.sparse-root.inverse.v1"),
                    DeclaredAftermathPostcondition::ExactPriorTruth,
                    demand,
                ),
            )),
            None,
        )
        .with_untrusted_correction_binding(
            DeclaredCorrectionBinding::from_untrusted_parts(ARCHIVE_DECODE, association),
        );
        let mut integrity = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        integrity.cardinality =
            ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
        let mut actual = vec![
            key.clone(),
            ApplicationSchemaMember::OperationAftermath {
                operation: binding.operation_name().to_owned(),
                contract,
            },
            ApplicationSchemaMember::Relation {
                relation: "MemberOf".into(),
                from: owner.from().to_owned(),
                to: owner.to().to_owned(),
                integrity: integrity.into(),
            },
        ];
        for target in [
            Target::Create {
                entity: entity.clone(),
            },
            Target::Delete {
                entity: entity.clone(),
            },
        ] {
            actual.push(ApplicationSchemaMember::OperationProgram {
                operation: binding.operation_name().to_owned(),
                target,
            });
        }
        let inventory = BTreeMap::from([(binding.identity().to_owned(), binding)]);
        let result = resolve_correction_binding(&actual, binding, &inventory, &queries);
        match denial {
            None => assert!(result.unwrap().is_some(), "{axis}"),
            Some(kind) => assert_eq!(result.unwrap_err().kind(), kind, "{axis}"),
        }
    }
}
