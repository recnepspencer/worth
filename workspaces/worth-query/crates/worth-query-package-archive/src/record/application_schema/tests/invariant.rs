use super::{decode_member, encode_member, RecordDecodeAttempt, WorthQueryPackageArchiveLimits};
use std::num::NonZeroU64;
use worth_query_declaration::facade::application_schema::{
    validate_portable_application_schema_freshly, ApplicationInvariantCostPosture as Cost,
    ApplicationInvariantEnforcement as Enforcement, ApplicationInvariantExecutionPoint as Point,
    ApplicationInvariantGroup as Group, ApplicationInvariantScopeTarget as Target,
    ApplicationRelationIntegrity, ApplicationSchemaMember,
    WorthQueryPortableApplicationSchemaParts, WorthQueryPortableApplicationSchemaRecord,
};

#[test]
fn invariant_archive_preserves_every_operational_axis_and_canonical_identity() {
    for (point, enforcement, cost) in [
        (
            Point::CommitBoundary,
            Enforcement::BlockCommit,
            Cost::Touched,
        ),
        (
            Point::MutationSensitive,
            Enforcement::BlockCommit,
            Cost::Partition,
        ),
        (
            Point::SnapshotPublication,
            Enforcement::BlockPublication,
            Cost::Global,
        ),
    ] {
        let member = invariant(point, enforcement, cost);
        let bytes = encode_member(&member);
        assert_eq!(&bytes[..2], &27u16.to_be_bytes());
        let decoded = decode_member(&bytes);
        assert_eq!(decoded, member);
        assert_eq!(encode_member(&decoded), bytes);
        let expected =
            validate_portable_application_schema_freshly(schema(member.clone())).unwrap();
        let restored = validate_portable_application_schema_freshly(schema(decoded)).unwrap();
        assert_eq!(expected.identity(), restored.identity());
        let mut changed = member;
        let ApplicationSchemaMember::ApplicationInvariant {
            maximum_work_units, ..
        } = &mut changed
        else {
            unreachable!()
        };
        *maximum_work_units = NonZeroU64::new(8193).unwrap();
        let changed = validate_portable_application_schema_freshly(schema(changed)).unwrap();
        assert_ne!(expected.identity(), changed.identity());
    }
}

#[test]
fn invariant_archive_rejects_unknown_checkpoint_and_zero_work() {
    let member = invariant(
        Point::CommitBoundary,
        Enforcement::BlockCommit,
        Cost::Touched,
    );
    let bytes = encode_member(&member);
    // Member tag, length-prefixed rule identity, major/minor precede checkpoint.
    let checkpoint = 2 + 4 + "ArchiveInvariant".len() + 2 + 2;
    let mut unknown = bytes.clone();
    unknown[checkpoint..checkpoint + 2].copy_from_slice(&99u16.to_be_bytes());
    assert_eq!(
        decode_denial(&unknown),
        crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
    let mut zero_work = bytes;
    zero_work[checkpoint + 2..checkpoint + 10].fill(0);
    assert_eq!(
        decode_denial(&zero_work),
        crate::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
}

fn decode_denial(bytes: &[u8]) -> crate::facade::WorthQueryPackageArchiveDenialKind {
    let mut input = crate::binary_input::BinaryInput::new(bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    super::super::member::decode(&mut input, &mut budget)
        .unwrap_err()
        .kind()
}

fn invariant(
    point: Point,
    enforcement: Enforcement,
    cost_posture: Cost,
) -> ApplicationSchemaMember {
    ApplicationSchemaMember::ApplicationInvariant {
        invariant: "ArchiveInvariant".to_owned(),
        major: 7,
        minor: 3,
        execution_point: point,
        maximum_work_units: NonZeroU64::new(8192).unwrap(),
        enforcement,
        required_groups: vec![
            Group::StorageCoherence,
            Group::VersionVisibility,
            Group::AdjacencyIntegrity,
            Group::IdentityCoherence,
            Group::SchemaCompliance,
            Group::LineageIntegrity,
            Group::PublicationCoherence,
            Group::RelationIntegrity,
        ],
        read_closure: vec![
            Target::Entity("Node".to_owned()),
            Target::Relation("Edge".to_owned()),
        ],
        applicability: vec![
            Target::Entity("Node".to_owned()),
            Target::Relation("Edge".to_owned()),
        ],
        provider: "primary-relational-provider".to_owned(),
        cost_posture,
    }
}

fn schema(invariant: ApplicationSchemaMember) -> WorthQueryPortableApplicationSchemaRecord {
    let mut members = vec![
        invariant,
        ApplicationSchemaMember::Entity {
            entity: "Node".to_owned(),
        },
        ApplicationSchemaMember::Relation {
            relation: "Edge".to_owned(),
            from: "Node".to_owned(),
            to: "Node".to_owned(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        },
    ];
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "archive.tests".to_owned(),
            name: "InvariantSchema".to_owned(),
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
        },
    )
}
