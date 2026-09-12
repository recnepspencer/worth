//! Real public record reads over the producer's acknowledged identities.
use super::{
    artifact_edit::ArtifactOperator as Op, artifact_inventory::ArtifactInventory,
    artifact_process::ArtifactRequest,
};
use worth_store::physical_runtime::{
    RecordByteLimit, RecordReadDenial, RecordReadLimits, ServingPhysicalRuntime,
};

pub(super) fn require(
    serving: &ServingPhysicalRuntime,
    request: &ArtifactRequest,
    inventory: &ArtifactInventory,
) {
    let target = request.poison.map(|index| &inventory.granules[index]);
    if let Some(target) = target {
        // Root-open, allocation planning and recovery own the other families;
        // a record read must not pretend to consume their bytes after reopen.
        if !matches!(
            target.family,
            "root_routing_block"
                | "segment_membership_block"
                | "inline_page"
                | "extent_manifest"
                | "extent_chunk"
        ) || !matches!(
            request.operator,
            Op::CoveredByte | Op::Checksum | Op::ScopeSubstitution
        ) {
            return;
        }
    }
    assert!(
        !request.records.is_empty(),
        "record identities must come from actual completed producer writes"
    );
    let before = serving.resident_admission_counters();
    let limits = RecordReadLimits::new(RecordByteLimit::new(1024 * 1024).unwrap());
    let mut refused = false;
    let mut completed = 0;
    for record in &request.records {
        let mut session = match serving
            .records()
            .expect("the single-reader fixture must admit root protection")
            .open_external(record.locator(inventory.store.bytes()), limits)
        {
            Ok(session) => session,
            Err(error) => {
                assert!(
                    target.is_some(),
                    "clean production record denied: {error:?}"
                );
                let expected = match (request.operator, target.unwrap().family) {
                    (Op::ScopeSubstitution, "inline_page") => RecordReadDenial::StalePlacement(
                        worth_store::physical_runtime::StalePhysicalRecordPlacement::PageIdentity),
                    (Op::ScopeSubstitution, "extent_manifest") => RecordReadDenial::StalePlacement(
                        worth_store::physical_runtime::StalePhysicalRecordPlacement::ExtentMembership),
                    _ => RecordReadDenial::ArtifactDamaged,
                };
                assert_eq!(error.denial(), expected, "{error:?}");
                refused = true;
                break;
            }
        };
        let mut scratch = [0; 4096];
        let mut length = 0;
        loop {
            let count = match session.read_next(&mut scratch) {
                Ok(count) => count,
                Err(error) => {
                    assert_eq!(target.unwrap().family, "extent_chunk", "{error:?}");
                    assert_eq!(
                        error.kind(),
                        if request.operator == Op::ScopeSubstitution {
                            worth_store::physical_runtime::RecordStreamFailureKind::StalePlacement
                        } else {
                            worth_store::physical_runtime::RecordStreamFailureKind::ArtifactDamaged
                        },
                        "{error:?}"
                    );
                    refused = true;
                    break;
                }
            };
            if count == 0 {
                break;
            }
            assert!(
                scratch[..count].iter().all(|byte| *byte == record.byte),
                "ordinary reader returned bytes from a different production record"
            );
            length += count;
            assert!(length <= record.length);
        }
        if refused {
            break;
        }
        assert_eq!(length, record.length);
        assert_eq!(session.observation().payload_bytes(), record.length as u64);
        completed += 1;
    }
    let after = serving.resident_admission_counters();
    assert_eq!(
        after.failed_rechecks_after_owner_entry(),
        before.failed_rechecks_after_owner_entry()
    );
    if target.is_some() {
        assert!(
            refused,
            "the actual record journey must encounter the declared poison"
        );
        assert_eq!(
            after.refusals_before_owner_entry() - before.refusals_before_owner_entry(),
            1
        );
        // Legitimate prerequisite tree decoders may enter before reaching the
        // poisoned page. The failed validation itself must add no owner entry.
        assert_eq!(
            after.fresh_validations() - before.fresh_validations() + after.exact_record_reuses() - before.exact_record_reuses(),
            after.owner_decoder_entries() - before.owner_decoder_entries() + after.owner_projection_entries() - before.owner_projection_entries() + 1,
            "exact successful-admission/owner conservation; rejected artifact enters neither owner lane"
        );
    } else {
        assert_eq!(completed, request.records.len());
        assert_eq!(
            after.refusals_before_owner_entry(),
            before.refusals_before_owner_entry()
        );
        assert!(after.fresh_validations() > before.fresh_validations());
        assert!(after.owner_decoder_entries() > before.owner_decoder_entries());
    }
    println!(
        "C9 ordinary records={} rejected={} counters={after:?}",
        completed, refused
    );
}
