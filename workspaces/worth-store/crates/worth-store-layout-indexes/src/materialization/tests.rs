use super::{CoverageBasisKind, MaterializationDenial};
use crate::facade::{access_planning, layout_declarations};
use crate::keyspace::tests_support::{
    published_blob_evidence_bundle, published_blob_import_declaration,
};
use crate::observation::AccessShape;
use crate::ExpectedCounterClass;
use std::collections::BTreeSet;
use worth_store_physical_format::CheckpointWalSourceRange;
use worth_store_physical_format::PhysicalEpoch;
use worth_store_wal::LogSequenceNumber;

fn format_family() -> &'static crate::PhysicalArtifactFamilyDeclaration {
    layout_declarations().seed_family()
}

fn imported_blob_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    crate::strategy::tests_support::admit_strategy_scope(
        worth_store_contracts::DurableArtifactFamilyId::BlobManifest,
        worth_store_security::StoreKeyScope::BlobChunkEnvelope,
        worth_store_security::StoreTenantScope::TenantPhysicalBoundary,
        worth_store_security::StoreAuthenticityRequirement::required(
            worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
        ),
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
    )
}

fn imported_blob_scope_for_store(
    store_authority_key: &str,
) -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    crate::strategy::tests_support::admit_strategy_scope_for_store(
        worth_store_contracts::DurableArtifactFamilyId::BlobManifest,
        worth_store_security::StoreKeyScope::BlobChunkEnvelope,
        worth_store_security::StoreTenantScope::TenantPhysicalBoundary,
        worth_store_security::StoreAuthenticityRequirement::required(
            worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
        ),
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
        store_authority_key,
    )
}

#[test]
fn imported_blob_materialization_retains_content_bound_owner_identity() {
    let witness =
        worth_store_blob_chunks::certification_test_authority::execute_readmitted_blob_import(
            "layout.import.materialization",
        );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let admission = access_planning().admit_imported_blob_materialization(
        imported_blob_scope().0,
        &catalog,
        &witness,
    );
    assert!(matches!(
        admission.view(),
        crate::ImportedBlobMaterializationAdmissionView::Admitted(_)
    ));
    let materialization = admission.expect("readmitted blob witness should materialize");

    assert_eq!(
        materialization.coverage().upper_bound().basis_kind(),
        CoverageBasisKind::BlobGeneration
    );
    assert_eq!(
        materialization.coverage().upper_bound().value(),
        witness.generation().sequence()
    );
    assert!(matches!(
        materialization.source().kind(),
        crate::LayoutMaterializationSourceKind::ImportedBlob(_)
    ));

    let other =
        worth_store_blob_chunks::certification_test_authority::execute_readmitted_blob_import(
            "layout.import.materialization.other",
        );
    let other_materialization = access_planning()
        .admit_imported_blob_materialization(imported_blob_scope().0, &catalog, &other)
        .expect("second readmitted blob witness should materialize");
    assert_ne!(materialization.source(), other_materialization.source());
}

#[test]
fn imported_blob_materialization_rejects_wrong_family() {
    let witness =
        worth_store_blob_chunks::certification_test_authority::execute_readmitted_blob_import(
            "layout.import.denial",
        );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let page_family = crate::strategy::tests_support::admit_btree_page_strategy().admitted_family();

    let admission =
        access_planning().admit_imported_blob_materialization(page_family, &catalog, &witness);
    assert!(matches!(
        admission.view(),
        crate::ImportedBlobMaterializationAdmissionView::Denied(
            MaterializationDenial::ImportedBlobFamilyRequired
        )
    ));
    assert_eq!(
        admission,
        Err(MaterializationDenial::ImportedBlobFamilyRequired)
    );
}

#[test]
fn imported_blob_materialization_rejects_another_store_authority() {
    let witness = worth_store_blob_chunks::certification_test_authority::
        execute_readmitted_blob_import_for_store(
            "layout.import.cross-store",
            "store.other.strategy",
        );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();

    assert_eq!(
        access_planning().admit_imported_blob_materialization(
            imported_blob_scope().0,
            &catalog,
            &witness,
        ),
        Err(MaterializationDenial::ImportedBlobStoreAuthorityMismatch)
    );
}

#[test]
fn imported_blob_witness_enters_read_planning_without_raw_identity_reconstruction() {
    let witness =
        worth_store_blob_chunks::certification_test_authority::execute_readmitted_blob_import(
            "layout.import.read-request",
        );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let (family, key_domain) = imported_blob_scope();

    let request = access_planning()
        .admit_imported_blob_read_request(family, key_domain, &catalog, &witness)
        .into_admitted()
        .expect("readmitted blob witness should issue a source-bound read request");

    assert!(matches!(
        request.materialization().source().kind(),
        crate::LayoutMaterializationSourceKind::ImportedBlob(_)
    ));
    assert_eq!(
        request.materialization().coverage().upper_bound().value(),
        witness.generation().sequence()
    );

    let selected = crate::planning::AccessPlanSelector
        .select_admitted_with_budget(
            request,
            worth_store_budgets::PreExecutionBudgetEnvelope::foreground_default(),
        )
        .into_lsm_lookup()
        .expect("blob manifest point lookup should select its admitted indexed owner");
    assert!(matches!(
        selected.materialization().source().kind(),
        crate::LayoutMaterializationSourceKind::ImportedBlob(_)
    ));
}

#[test]
fn imported_blob_read_admission_declares_exactly_ordinary_owner_cases() {
    let witness =
        worth_store_blob_chunks::certification_test_authority::execute_readmitted_blob_import(
            "layout.import.case-coverage",
        );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let (family, key_domain) = imported_blob_scope();
    let page = crate::strategy::tests_support::admit_btree_page_strategy();
    let (_, other_blob_domain) = imported_blob_scope_for_store("store.other.strategy");

    let observed = [
        access_planning().admit_imported_blob_read_request(family, key_domain, &catalog, &witness),
        access_planning().admit_imported_blob_read_request(
            page.admitted_family(),
            page.admitted_key_domain(),
            &catalog,
            &witness,
        ),
        access_planning().admit_imported_blob_read_request(
            family,
            page.admitted_key_domain(),
            &catalog,
            &witness,
        ),
        access_planning().admit_imported_blob_read_request(
            family,
            other_blob_domain,
            &catalog,
            &witness,
        ),
    ]
    .into_iter()
    .map(|outcome| outcome.case_id())
    .collect::<BTreeSet<_>>();

    assert_eq!(
        observed,
        crate::imported_blob_read_admission_cases().collect::<BTreeSet<_>>()
    );
}

#[test]
fn stale_coverage_cannot_become_current_materialization() {
    let family = format_family().family();
    let stale = access_planning()
        .stale_root_epoch_coverage(
            format_family(),
            PhysicalEpoch::from_raw(7).expect("epoch fixture should be valid"),
        )
        .expect("coverage should build");

    assert_eq!(
        stale.require_exact(),
        Err(MaterializationDenial::LayoutCoverageIsStale {
            family,
            basis_kind: CoverageBasisKind::RootEpoch,
        })
    );
}

#[test]
fn partial_coverage_localizes_gap() {
    let gap = CheckpointWalSourceRange::new(11, 19).expect("gap fixture should be valid");
    let partial = access_planning()
        .partial_wal_lsn_coverage(
            format_family(),
            LogSequenceNumber::new(10),
            LogSequenceNumber::new(20),
            gap,
        )
        .expect("partial coverage should build");

    assert_eq!(
        partial.require_exact(),
        Err(MaterializationDenial::LayoutCoverageIsPartial {
            gap: partial.gap().expect("partial coverage retains its gap"),
        })
    );
}

#[test]
fn exact_through_basis_survives_range_and_prefix_completeness() {
    let coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            PhysicalEpoch::from_raw(31).expect("epoch fixture should be valid"),
        )
        .expect("exact coverage should build");

    coverage.require_exact().expect("coverage should be exact");
    let range = access_planning().range_access();
    let prefix = access_planning().prefix_access();

    assert_eq!(range.shape(), AccessShape::RangeLookup);
    assert_eq!(range.expected_counters(), ExpectedCounterClass::RangeLookup);
    assert_eq!(prefix.shape(), AccessShape::PrefixLookup);
    assert_eq!(
        prefix.expected_counters(),
        ExpectedCounterClass::PrefixLookup
    );
}

#[test]
fn checkpoint_and_blob_generation_coverages_are_first_class_public_lanes() {
    let checkpoint =
        CheckpointWalSourceRange::new(21, 29).expect("checkpoint fixture should be valid");
    let checkpoint_coverage = access_planning()
        .exact_checkpoint_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            checkpoint,
        )
        .expect("checkpoint coverage should admit");
    assert_eq!(
        checkpoint_coverage.upper_bound().basis_kind(),
        CoverageBasisKind::CheckpointFrontier
    );
    assert_eq!(checkpoint_coverage.upper_bound().start_inclusive(), 21);
    assert_eq!(checkpoint_coverage.upper_bound().value(), 29);

    let blob_bundle = published_blob_evidence_bundle();
    let blob_coverage = access_planning()
        .exact_blob_generation_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            crate::BlobGenerationBasis::from_sequence(blob_bundle.export_generation().sequence()),
        )
        .expect("blob generation coverage should admit");
    assert_eq!(
        blob_coverage.upper_bound().basis_kind(),
        CoverageBasisKind::BlobGeneration
    );
}

#[test]
fn coverage_basis_witnesses_survive_reopen_and_certification_replay() {
    let root_epoch = PhysicalEpoch::from_raw(37).expect("epoch fixture should be valid");
    let root_from_open = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            root_epoch,
        )
        .expect("root epoch coverage should admit");
    let root_from_reopen = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            root_epoch,
        )
        .expect("reopened root epoch coverage should admit");

    assert_eq!(root_from_open, root_from_reopen);

    let wal_lsn = LogSequenceNumber::new(64);
    let wal_from_log = access_planning()
        .exact_wal_lsn_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            wal_lsn,
        )
        .expect("wal lsn coverage should admit");
    let wal_from_replay = access_planning()
        .exact_wal_lsn_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            wal_lsn,
        )
        .expect("replayed wal lsn coverage should admit");

    assert_eq!(wal_from_log, wal_from_replay);

    let blob_bundle = published_blob_evidence_bundle();
    let import_declaration = published_blob_import_declaration();
    let blob_from_lifecycle = access_planning()
        .exact_blob_generation_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            crate::BlobGenerationBasis::from_sequence(
                blob_bundle.lifecycle_declaration().generation().sequence(),
            ),
        )
        .expect("lifecycle blob generation should admit");
    let blob_from_export = access_planning()
        .exact_blob_generation_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            crate::BlobGenerationBasis::from_sequence(blob_bundle.export_generation().sequence()),
        )
        .expect("export blob generation should admit");
    let blob_from_replay = access_planning()
        .exact_blob_generation_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            crate::BlobGenerationBasis::from_sequence(import_declaration.generation().sequence()),
        )
        .expect("replayed blob generation should admit");

    assert_eq!(blob_from_lifecycle, blob_from_export);
    assert_eq!(blob_from_lifecycle, blob_from_replay);
}
