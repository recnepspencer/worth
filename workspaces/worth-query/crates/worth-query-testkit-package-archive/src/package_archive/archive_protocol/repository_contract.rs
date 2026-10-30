pub use std::collections::BTreeMap;
pub use std::sync::Mutex;

pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainPackageIdentity,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
};
pub use worth_query_package_archive::facade::*;

pub use super::archive_stream::fixture as package_fixture;
pub use super::release_envelope_fixture::*;

#[derive(Default)]
pub struct MemoryPackageArchiveRepository {
    pub records: Mutex<BTreeMap<WorthQueryPortableDomainPackageIdentity, Vec<u8>>>,
}

impl WorthQueryPackageArchiveRepository for MemoryPackageArchiveRepository {
    fn store_exact(
        &self,
        record: &WorthQuerySignedPackageArchiveRecord,
    ) -> WorthQueryPackageArchiveStoreOutcome {
        let mut records = self.records.lock().unwrap();
        match records.get(record.claimed_package_identity()) {
            Some(bytes) if bytes == record.exact_envelope_bytes() => {
                WorthQueryPackageArchiveStoreOutcome::AlreadyStoredExact
            }
            Some(_) => WorthQueryPackageArchiveStoreOutcome::IdentityConflict(
                WorthQueryPackageArchiveIdentityConflict::new(
                    record.claimed_package_identity().clone(),
                ),
            ),
            None => {
                records.insert(
                    record.claimed_package_identity().clone(),
                    record.exact_envelope_bytes().to_vec(),
                );
                WorthQueryPackageArchiveStoreOutcome::Stored
            }
        }
    }

    fn load_exact(
        &self,
        request: WorthQueryExactPackageArchiveRequest,
    ) -> WorthQueryPackageArchiveLoadOutcome {
        let records = self.records.lock().unwrap();
        let Some(bytes) = records.get(request.expected_package_identity()) else {
            return WorthQueryPackageArchiveLoadOutcome::NotFound;
        };
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX)
            > request.envelope_limits().maximum_envelope_bytes()
        {
            return WorthQueryPackageArchiveLoadOutcome::Denied(
                WorthQueryPackageArchiveRepositoryDenial::new(
                    WorthQueryPackageArchiveRepositoryDenialKind::EnvelopeByteBudgetExceeded,
                ),
            );
        }
        WorthQueryPackageArchiveLoadOutcome::Found(
            WorthQueryUntrustedLoadedPackageArchive::from_untrusted_bytes(request, bytes.clone())
                .unwrap(),
        )
    }
}

pub fn record_for_minimal_package(signature_byte: u8) -> WorthQuerySignedPackageArchiveRecord {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    signed_record(&records, signature_byte)
}

pub fn record_for_all_family_package(signature_byte: u8) -> WorthQuerySignedPackageArchiveRecord {
    let records = package_fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    signed_record(&records, signature_byte)
}

pub fn signed_record(
    records: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
    signature_byte: u8,
) -> WorthQuerySignedPackageArchiveRecord {
    let envelope = unsigned_envelope(records, fixture_descriptor())
        .attach_signature(
            signature(signature_byte),
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap();
    WorthQuerySignedPackageArchiveRecord::from_signed_envelope(
        envelope,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap()
}

pub fn loaded(
    repository: &MemoryPackageArchiveRepository,
    identity: WorthQueryPortableDomainPackageIdentity,
) -> WorthQueryUntrustedLoadedPackageArchive {
    match repository.load_exact(WorthQueryExactPackageArchiveRequest::new(
        identity,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )) {
        WorthQueryPackageArchiveLoadOutcome::Found(loaded) => loaded,
        outcome => panic!("exact archive was not found: {outcome:?}"),
    }
}

pub fn loaded_bytes(
    repository: &MemoryPackageArchiveRepository,
    identity: WorthQueryPortableDomainPackageIdentity,
) -> Vec<u8> {
    loaded(repository, identity).into_untrusted_envelope_bytes()
}

pub fn release_name(record: &WorthQuerySignedPackageArchiveRecord) -> String {
    decode_package_release_envelope(
        record.exact_envelope_bytes(),
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap()
    .envelope()
    .unsigned()
    .release_metadata()
    .release_name()
    .to_owned()
}
