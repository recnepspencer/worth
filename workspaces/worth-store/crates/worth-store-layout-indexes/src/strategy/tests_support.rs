use crate::strategy::registry::{
    layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
};
use crate::strategy::{AdmittedLayoutStrategy, LayoutStrategyFamily};
use crate::{
    layout_declarations, AdmittedPhysicalArtifactFamily, AdmittedPhysicalKeyDomain,
    ArtifactFamilyAccessLane, CanonicalKeyBytes,
};
use worth_foundational::{aspects, AspectContract, AspectValue, InternedString, ScalarAspectType};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalBoundaryWitness,
};
use worth_store_authority::{require_current_store_authority, StoreCurrentAuthorityWitness};
use worth_store_contracts::{
    DurableArtifactFamilyId, StorePhysicalAuthorityWitness, WalRecordFamily,
    ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
use worth_store_physical_format::{PhysicalPageId, PhysicalSegmentId};
use worth_store_security::{
    admit_store_security_scope, StoreAdmittedSecurityScope, StoreAuthenticityRequirement,
    StoreAuthenticityRequirementClass, StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};
use worth_store_wal::StoreWalRecordIdentity;

pub(crate) fn admit_strategy_scope(
    family_id: DurableArtifactFamilyId,
    key_scope: StoreKeyScope,
    tenant_scope: StoreTenantScope,
    authenticity: StoreAuthenticityRequirement,
    custody: StoreCustodyPosture,
) -> (AdmittedPhysicalArtifactFamily, AdmittedPhysicalKeyDomain) {
    admit_strategy_scope_for_store(
        family_id,
        key_scope,
        tenant_scope,
        authenticity,
        custody,
        strategy_store_authority_key(),
    )
}

pub(crate) fn strategy_test_store_identity() -> worth_store_physical_format::PhysicalStoreIdentity {
    let key = worth_foundational::aspects()
        .vocabulary()
        .key(strategy_store_authority_key())
        .expect("strategy test Store identity key");
    worth_store_physical_format::PhysicalStoreIdentity::from_aspect_identity(
        StoreAspectIdentity::from_aspect_key(key),
    )
}

pub(crate) fn strategy_test_wal_security_scope_for_store(
    store_authority_key: &str,
) -> StoreAdmittedSecurityScope {
    admitted_scope(
        StoreKeyScope::WalCheckpointEnvelope,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedWalRecord,
        ),
        StoreCustodyPosture::InternalStoreCustody,
        store_authority_key,
    )
}

const fn strategy_store_authority_key() -> &'static str {
    "store.new.strategy"
}

pub(crate) fn admit_strategy_scope_for_store(
    family_id: DurableArtifactFamilyId,
    key_scope: StoreKeyScope,
    tenant_scope: StoreTenantScope,
    authenticity: StoreAuthenticityRequirement,
    custody: StoreCustodyPosture,
    store_authority_key: &str,
) -> (AdmittedPhysicalArtifactFamily, AdmittedPhysicalKeyDomain) {
    let security_scope = admitted_scope(
        key_scope,
        tenant_scope,
        authenticity,
        custody,
        store_authority_key,
    );
    let declaration = layout_declarations().declaration(family_id).unwrap();
    let family = layout_declarations()
        .admit_physical_artifact_family(declaration, security_scope.witnesses())
        .unwrap();
    let key_domain = layout_declarations()
        .admit_physical_key_domain(family, security_scope.witnesses())
        .unwrap();
    (family, key_domain)
}

pub(crate) fn admit_btree_page_strategy() -> AdmittedLayoutStrategy {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    *layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            lifecycle,
            key_domain,
            LayoutStrategyFamily::BTreeRange,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .unwrap()
        .admitted_strategy()
}

pub(crate) fn admit_lsm_wal_strategy() -> AdmittedLayoutStrategy {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PublicationWalIntent,
        StoreKeyScope::WalCheckpointEnvelope,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedWalRecord,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    *layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            lifecycle,
            key_domain,
            LayoutStrategyFamily::BaselineLsmWriteOptimized,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .unwrap()
        .admitted_strategy()
}

pub(crate) fn admit_persisted_lsm_strategy() -> AdmittedLayoutStrategy {
    let (family, key_domain) = admit_persisted_lsm_scope();
    *layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            family,
            key_domain,
            LayoutStrategyFamily::BaselineLsmWriteOptimized,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .unwrap()
        .admitted_strategy()
}

pub(crate) fn admit_persisted_lsm_scope(
) -> (AdmittedPhysicalArtifactFamily, AdmittedPhysicalKeyDomain) {
    let security =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    let declaration = layout_declarations()
        .declaration(DurableArtifactFamilyId::PublicationWalIntent)
        .unwrap();
    let family = layout_declarations()
        .admit_physical_artifact_family(declaration, security.witnesses())
        .unwrap();
    let key_domain = layout_declarations()
        .admit_physical_key_domain(family, security.witnesses())
        .unwrap();
    (family, key_domain)
}

pub(crate) fn persisted_lsm_materialization(
    family: AdmittedPhysicalArtifactFamily,
    catalog: &crate::BootstrapCatalogReadAdmission,
) -> (
    crate::AdmittedLayoutMaterialization,
    crate::BaselineLsmLookupSource,
) {
    let replacement = certification_published_lsm_membership_replacement();
    let source = crate::lsm_strategy()
        .readmit_lookup_source(family, &replacement)
        .expect("persisted LSM membership must readmit under its exact Store family");
    let materialization = crate::access_planning()
        .admit_lsm_lookup_materialization(family, catalog, &source)
        .expect("persisted LSM owner source must admit exact materialization");
    (materialization, source)
}

pub(crate) fn certification_published_lsm_membership_replacement(
) -> worth_store_lsm_authority::PublishedLsmMembershipReplacement {
    let security =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    let metadata = worth_store_wal::WalSecurityMetadataCarrier::for_wal_record(
        security.witnesses(),
        StoreKeyVersionPosture::Current,
        worth_store_security::StoreLegacySecurityPosture::NativeScoped,
    );
    let key =
        worth_store_lsm_authority::LsmMembershipKey::admit(metadata, b"certification-layout-index")
            .expect("certification LSM key is canonical");
    worth_store_lsm_authority::issue_published_lsm_membership_for_certification(key)
}

pub(crate) fn admitted_page_key_bytes(segment: u64, page: u64) -> CanonicalKeyBytes {
    let strategy = admit_btree_page_strategy();
    let domain = strategy.key_domain();
    let encoding = layout_declarations().require_canonical_key_encoding(domain);
    let comparator = layout_declarations().declare_comparator_law(encoding);
    let key = layout_declarations()
        .admit_page_address_key(domain, segment_id(segment), page_id(page))
        .unwrap();
    layout_declarations()
        .canonical_key_bytes(comparator, key)
        .unwrap()
}

pub(crate) fn admitted_wal_key_bytes(sequence: u64) -> CanonicalKeyBytes {
    let strategy = admit_lsm_wal_strategy();
    let domain = strategy.key_domain();
    let encoding = layout_declarations().require_canonical_key_encoding(domain);
    let comparator = layout_declarations().declare_comparator_law(encoding);
    let key = layout_declarations()
        .admit_wal_record_key(
            domain,
            WalRecordFamily::DurableMutationIntent,
            StoreWalRecordIdentity::new(sequence),
        )
        .unwrap();
    layout_declarations()
        .canonical_key_bytes(comparator, key)
        .unwrap()
}

pub(crate) fn root_manifest_scope() -> (AdmittedPhysicalArtifactFamily, AdmittedPhysicalKeyDomain) {
    admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalRootManifest,
        StoreKeyScope::StoreManagedRoot,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::not_required(),
        StoreCustodyPosture::InternalStoreCustody,
    )
}

fn admitted_scope(
    key_scope: StoreKeyScope,
    tenant_scope: StoreTenantScope,
    authenticity_requirement: StoreAuthenticityRequirement,
    custody_posture: StoreCustodyPosture,
    store_authority_key: &str,
) -> StoreAdmittedSecurityScope {
    let current_authority = current_authority(store_authority_key, "test-current");
    let expectation = StoreSecurityScopeAdmissionExpectation::new(
        key_scope,
        tenant_scope,
        authenticity_requirement,
        custody_posture,
    );
    let request = StoreSecurityScopeAdmissionRequest::new(
        &current_authority,
        key_scope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity_requirement,
        custody_posture,
        expectation,
    );

    match admit_store_security_scope(request) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("strategy scope admission should succeed: {outcome:?}"),
    }
}

fn current_authority(identity_key: &str, value: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(identity_key, value))
}

fn boundary_fact(identity_key: &str, value: &str) -> StoreAspectBoundaryFact {
    let key = aspects().vocabulary().key(identity_key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String);
    let admitted_state = match aspects()
        .authoritative_state()
        .admit([validated_scalar_value(&contract, value)])
    {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };

    StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key),
        StoreAspectAuthorityInput::new(admitted_state, physical_witness()),
    )
    .expect("Store boundary fact should admit matching identity")
}

fn validated_scalar_value(
    contract: &AspectContract,
    raw_value: &str,
) -> worth_foundational::ContractValidatedAspectArtifact {
    match aspects()
        .validate()
        .against(contract)
        .value(AspectValue::String(InternedString::from(raw_value)))
    {
        TransitionOutcome::Success(value) => value,
        outcome => panic!("validation should succeed: {outcome:?}"),
    }
}

fn physical_witness() -> StorePhysicalBoundaryWitness {
    StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .expect("test physical authority scope should be valid"),
    )
    .expect("test physical boundary witness should admit")
}

fn segment_id(value: u64) -> PhysicalSegmentId {
    PhysicalSegmentId::from_raw(value).expect("test segment id is non-zero")
}

fn page_id(value: u64) -> PhysicalPageId {
    PhysicalPageId::from_raw(value).expect("test page id is non-zero")
}
