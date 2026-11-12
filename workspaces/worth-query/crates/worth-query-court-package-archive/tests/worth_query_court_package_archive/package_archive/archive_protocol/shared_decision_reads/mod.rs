//! Public archive bytes preserve the real SDK package and exact scoped meaning.
use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::shared_decision_reads::*;
mod limits;

#[test]
fn shared_scope_export_archive_decode_and_fresh_readmission_preserve_complete_meaning() {
    let (source, scope) = package(false);
    let (reordered, reordered_scope) = package(true);
    assert_eq!(
        scope.identity(),
        reordered_scope.identity(),
        "lawful authoring order is not identity"
    );
    assert_eq!(source.identity(), reordered.identity());
    let expected = source.identity().clone();
    let exported = source.export_typed_records().unwrap();
    let bytes = encode_package_archive(&exported, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let decoded = decode_package_archive(&bytes, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(decoded.manifest(), exported.manifest());
    let records = decoded
        .frames()
        .iter()
        .map(|frame| frame.record().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        records,
        exported.records(),
        "all exported meaning survives production transport"
    );
    let mut forms = [false; 3];
    for record in &records {
        match record {
            WorthQueryPortablePackageRecord::ApplicationSchema(schema) => {
                for member in schema.members() {
                    match member {
                        ApplicationSchemaMember::DecisionReadScope { definition } => {
                            assert_eq!(definition, &scope);
                            forms[0] = true;
                        }
                        ApplicationSchemaMember::OperationDecisionReadScope {
                            scope: identity,
                            ..
                        } => {
                            assert_eq!(identity, scope.identity());
                            forms[1] = true;
                        }
                        _ => {}
                    }
                }
            }
            WorthQueryPortablePackageRecord::ApplicationOperationContract(operation) => {
                for read in operation.graph_reads() {
                    if let WorthQueryPortableOperationGraphReadScope::SharedDecisionRead {
                        identity,
                        ..
                    } = read
                    {
                        assert_eq!(identity, scope.identity());
                        forms[2] = true;
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(forms, [true; 3]);
    let mut intake = WorthQueryPortablePackageReconstruction::begin(
        decoded.manifest().clone(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for frame in decoded.frames() {
        intake = intake
            .push_record(frame.canonical_index(), frame.record().clone())
            .unwrap();
    }
    let validated = intake
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(expected.clone()),
        )
        .unwrap();
    assert_eq!(validated.identity(), &expected);
}

#[test]
fn displaced18_refuses_before_poisoned_first_frame_content_while19_reaches_it() {
    let (source, _) = package(false);
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut bytes = encode_package_archive(&exported, limits).unwrap();
    assert!(
        decode_package_archive(&bytes, limits).is_ok(),
        "original enclosing manifest/inventory is valid"
    );
    let first = encode_manifest_frame(exported.manifest(), limits)
        .unwrap()
        .len();
    assert_eq!(
        &bytes[first + 2..first + 4],
        &1_u16.to_be_bytes(),
        "first frame is domain identity"
    );
    // Poison the first payload's owner-text length; preserve the valid enclosing inventory.
    bytes[first + 12..first + 16].copy_from_slice(&u32::MAX.to_be_bytes());
    let current = decode_package_archive(&bytes, limits).unwrap_err();
    assert_eq!(
        current.kind(),
        WorthQueryPackageArchiveDenialKind::Truncated
    );
    bytes[first..first + 2].copy_from_slice(&18_u16.to_be_bytes());
    let displaced = decode_package_archive(&bytes, limits).unwrap_err();
    assert_eq!(
        displaced.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    let compatibility = displaced.compatibility().unwrap();
    assert_eq!(
        compatibility.layer(),
        WorthQueryPackageArchiveProtocolLayer::RecordFrame
    );
    assert_eq!(compatibility.observed_version(), 18);
    super::protocol_compatibility::assert_window(compatibility.supported_window(), 19, 19);
}
