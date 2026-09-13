use super::readmission::{
    RecoveryLayoutReadmissionClass, RecoveryLayoutReadmissionIdentity,
    RecoveryLayoutReadmissionWitness,
};
use worth_foundational::{aspects, AspectContract, AspectValue, InternedString, ScalarAspectType};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalBoundaryWitness,
};
use worth_store_authority::{require_current_store_authority, StoreCurrentAuthorityWitness};
use worth_store_contracts::{
    StableDigest, StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
use worth_store_security::{
    admit_store_security_scope, StoreAdmittedSecurityScope, StoreAuthenticityRequirement,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LayoutQuarantineObservationFixture {
    identity: RecoveryLayoutReadmissionIdentity,
    class: RecoveryLayoutReadmissionClass,
}

impl LayoutQuarantineObservationFixture {
    pub(super) const fn identity(&self) -> &RecoveryLayoutReadmissionIdentity {
        &self.identity
    }

    pub(super) const fn class(&self) -> RecoveryLayoutReadmissionClass {
        self.class
    }
}

pub(super) fn import_witness(
    family: crate::PhysicalArtifactFamily,
    seed: &str,
) -> RecoveryLayoutReadmissionWitness {
    let observation = unresolved_authority_observation(seed);
    let authority = current_authority("store.new.strategy", seed);
    let security = current_security_scope("store.new.strategy", seed);
    super::readmission::layout_readmission()
        .admit_import(
            family.id(),
            observation.identity(),
            observation.class(),
            &authority,
            security.witnesses(),
        )
        .expect("observation-bound import witness should admit")
}

pub(super) fn quarantine_witness(
    family: crate::PhysicalArtifactFamily,
    seed: &str,
) -> RecoveryLayoutReadmissionWitness {
    let observation = authoritative_quarantine_observation(seed);
    observation_bound_witness(family, &observation, seed)
}

pub(super) fn observation_bound_witness(
    family: crate::PhysicalArtifactFamily,
    observation: &LayoutQuarantineObservationFixture,
    seed: &str,
) -> RecoveryLayoutReadmissionWitness {
    observation_bound_witness_for_store(family, observation, "store.new.strategy", seed)
}

pub(super) fn observation_bound_witness_for_store(
    family: crate::PhysicalArtifactFamily,
    observation: &LayoutQuarantineObservationFixture,
    store_authority_key: &str,
    seed: &str,
) -> RecoveryLayoutReadmissionWitness {
    observation_bound_witness_for_scope(
        family,
        observation,
        store_authority_key,
        seed,
        StoreKeyScope::StoreManagedRoot,
        StoreTenantScope::StoreInternal,
    )
}

pub(super) fn observation_bound_witness_for_scope(
    family: crate::PhysicalArtifactFamily,
    observation: &LayoutQuarantineObservationFixture,
    store_authority_key: &str,
    seed: &str,
    key_scope: StoreKeyScope,
    tenant_scope: StoreTenantScope,
) -> RecoveryLayoutReadmissionWitness {
    let authority = current_authority(store_authority_key, seed);
    let security = current_security_scope_with(store_authority_key, seed, key_scope, tenant_scope);
    super::readmission::layout_readmission()
        .admit_quarantine(
            family.id(),
            observation.identity(),
            observation.class(),
            &authority,
            security.witnesses(),
        )
        .expect("observation-bound quarantine witness should admit")
}

pub(super) fn authoritative_quarantine_observation(
    seed: &str,
) -> LayoutQuarantineObservationFixture {
    observation(seed, RecoveryLayoutReadmissionClass::QuarantineRecovery)
}

pub(super) fn unresolved_authority_observation(seed: &str) -> LayoutQuarantineObservationFixture {
    observation(
        seed,
        RecoveryLayoutReadmissionClass::ImportBoundaryReadmission,
    )
}

pub(super) fn current_authority(identity_key: &str, value: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(identity_key, value))
}

pub(super) fn current_security_scope(
    identity_key: &str,
    value: &str,
) -> StoreAdmittedSecurityScope {
    current_security_scope_with(
        identity_key,
        value,
        StoreKeyScope::StoreManagedRoot,
        StoreTenantScope::StoreInternal,
    )
}

pub(super) fn current_security_scope_with(
    identity_key: &str,
    value: &str,
    key_scope: StoreKeyScope,
    tenant_scope: StoreTenantScope,
) -> StoreAdmittedSecurityScope {
    let authority = current_authority(identity_key, value);
    let authenticity = StoreAuthenticityRequirement::not_required();
    let custody = StoreCustodyPosture::InternalStoreCustody;
    let expectation =
        StoreSecurityScopeAdmissionExpectation::new(key_scope, tenant_scope, authenticity, custody);
    match admit_store_security_scope(StoreSecurityScopeAdmissionRequest::new(
        &authority,
        key_scope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    )) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("current test security scope should admit: {outcome:?}"),
    }
}

fn observation(
    seed: &str,
    class: RecoveryLayoutReadmissionClass,
) -> LayoutQuarantineObservationFixture {
    let digest = StableDigest::new(format!("sha256:layout-quarantine:{seed}"))
        .expect("test observation digest is non-empty");
    LayoutQuarantineObservationFixture {
        identity: RecoveryLayoutReadmissionIdentity::QuarantineObservation(digest),
        class,
    }
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
