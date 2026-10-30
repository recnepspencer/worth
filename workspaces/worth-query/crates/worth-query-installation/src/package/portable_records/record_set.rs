//! Canonical assembly and exact source-closure validation for typed exports.

#[cfg(test)]
mod tests;

use super::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableNativeAspectContractRecord, WorthQueryPortablePackageExportDenial,
    WorthQueryPortablePackageExportDenialKind as DenialKind, WorthQueryPortablePackageExportLimits,
    WorthQueryPortablePackageManifest, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily, WorthQueryPortablePackageRecordView,
    WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
};
use crate::package::WorthQueryValidatedPortableDomainPackage;

const LOGICAL_RECORD_FAMILY_TAG_BYTES: u64 = 1;
const LOGICAL_RECORD_CANONICAL_INDEX_BYTES: u64 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortablePackageRecordSet {
    manifest: WorthQueryPortablePackageManifest,
    records: Vec<WorthQueryPortablePackageRecord>,
}

impl WorthQueryPortablePackageRecordSet {
    pub const fn manifest(&self) -> &WorthQueryPortablePackageManifest {
        &self.manifest
    }

    pub fn records(&self) -> &[WorthQueryPortablePackageRecord] {
        &self.records
    }

    pub fn views(&self) -> impl ExactSizeIterator<Item = WorthQueryPortablePackageRecordView<'_>> {
        self.records.iter().enumerate().map(|(index, record)| {
            WorthQueryPortablePackageRecordView::new(
                u32::try_from(index).expect("manifest already bounded record count"),
                record,
            )
        })
    }
}

pub(in crate::package) fn export_validated_package_records(
    package: &WorthQueryValidatedPortableDomainPackage,
    limits: WorthQueryPortablePackageExportLimits,
) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryPortablePackageExportDenial> {
    let maximum_records = limits
        .maximum_records()
        .min(WorthQueryPortablePackageExportLimits::DEFAULT.maximum_records());
    let maximum_logical_export_bytes = limits
        .maximum_logical_export_bytes()
        .min(WorthQueryPortablePackageExportLimits::DEFAULT.maximum_logical_export_bytes());
    let family_counts = expected_family_counts(package)?;
    let expected_count = family_counts
        .iter()
        .try_fold(0_u32, |total, count| total.checked_add(*count))
        .ok_or_else(|| denial(DenialKind::RecordCountExceeded))?;
    if expected_count > maximum_records {
        return Err(denial(DenialKind::RecordCountExceeded));
    }
    let canonical_source_bytes = u64::try_from(package.canonical_work().canonical_encoded_bytes())
        .map_err(|_| denial(DenialKind::LogicalExportBytesExceeded))?;
    let logical_export_bytes = logical_export_bytes(
        package,
        expected_count,
        canonical_source_bytes,
        maximum_logical_export_bytes,
    )?;

    let capacity =
        usize::try_from(expected_count).map_err(|_| denial(DenialKind::RecordCountExceeded))?;
    let records = assemble_records(package, capacity);
    let manifest = WorthQueryPortablePackageManifest::new(
        package.identity().clone(),
        expected_count,
        canonical_source_bytes,
        logical_export_bytes,
        family_counts,
    );
    validate_source_closure(package, &manifest, &records)?;
    Ok(WorthQueryPortablePackageRecordSet { manifest, records })
}

fn assemble_records(
    package: &WorthQueryValidatedPortableDomainPackage,
    capacity: usize,
) -> Vec<WorthQueryPortablePackageRecord> {
    let mut records = Vec::with_capacity(capacity);
    records.push(WorthQueryPortablePackageRecord::DomainIdentity(
        package.domain_identity().clone(),
    ));
    records.extend(
        package
            .capabilities()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::CapabilityRequirement),
    );
    records.extend(
        package
            .configuration()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::ConfigurationRequirement),
    );
    records.extend(
        package
            .operating_requirements()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::OperatingRequirement),
    );
    records.extend(
        package
            .definitions()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::Definition),
    );
    records.extend(
        package
            .domain_operations()
            .iter()
            .map(crate::package::WorthQueryPortableDomainOperationRecord::project)
            .map(WorthQueryPortablePackageRecord::DomainOperation),
    );
    records.extend(
        package
            .artifact_contracts()
            .iter()
            .map(crate::domain_computation::WorthQueryPortableArtifactContractRecord::project)
            .map(WorthQueryPortablePackageRecord::ArtifactContract),
    );
    records.extend(
        package
            .application_schemas()
            .iter()
            .map(
                worth_query_declaration::facade::application_schema::WorthQueryPortableApplicationSchemaRecord::project,
            )
            .map(WorthQueryPortablePackageRecord::ApplicationSchema),
    );
    records.extend(
        package
            .conditional_application_operations()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::ConditionalApplicationOperation),
    );
    records.extend(
        package
            .contribution_policy()
            .iter()
            .cloned()
            .map(WorthQueryPortablePackageRecord::ContributionPolicy),
    );
    records.extend(
        canonical_native_records(package)
            .into_iter()
            .map(WorthQueryPortablePackageRecord::NativeAspectContract),
    );
    records.extend(
        canonical_operation_records(package)
            .into_iter()
            .map(WorthQueryPortablePackageRecord::ApplicationOperationContract),
    );
    records
}

fn expected_family_counts(
    package: &WorthQueryValidatedPortableDomainPackage,
) -> Result<
    [u32; WorthQueryPortablePackageRecordFamily::ALL.len()],
    WorthQueryPortablePackageExportDenial,
> {
    let lengths = [
        1,
        package.capabilities().len(),
        package.configuration().len(),
        package.operating_requirements().len(),
        package.definitions().len(),
        package.domain_operations().len(),
        package.artifact_contracts().len(),
        package.application_schemas().len(),
        package.conditional_application_operations().len(),
        package.contribution_policy().len(),
        package.application_contract_spine().native_aspects().len(),
        package.application_contract_spine().operations().len(),
    ];
    let mut counts = [0_u32; WorthQueryPortablePackageRecordFamily::ALL.len()];
    for (target, length) in counts.iter_mut().zip(lengths) {
        *target = u32::try_from(length).map_err(|_| denial(DenialKind::RecordCountExceeded))?;
    }
    Ok(counts)
}

fn canonical_native_records(
    package: &WorthQueryValidatedPortableDomainPackage,
) -> Vec<WorthQueryPortableNativeAspectContractRecord> {
    let mut records = package
        .application_contract_spine()
        .native_aspects()
        .to_vec();
    records.sort_by(|left, right| {
        (left.schema(), left.entity(), left.aspect()).cmp(&(
            right.schema(),
            right.entity(),
            right.aspect(),
        ))
    });
    records
}

fn canonical_operation_records(
    package: &WorthQueryValidatedPortableDomainPackage,
) -> Vec<WorthQueryPortableApplicationOperationContractRecord> {
    let mut records = package.application_contract_spine().operations().to_vec();
    records.sort_by(|left, right| {
        (left.schema(), left.operation(), left.input_type()).cmp(&(
            right.schema(),
            right.operation(),
            right.input_type(),
        ))
    });
    records
}

fn validate_source_closure(
    package: &WorthQueryValidatedPortableDomainPackage,
    manifest: &WorthQueryPortablePackageManifest,
    records: &[WorthQueryPortablePackageRecord],
) -> Result<(), WorthQueryPortablePackageExportDenial> {
    let expected_counts = expected_family_counts(package)?;
    if manifest.version() != WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION
        || manifest.package_identity() != package.identity()
        || manifest.family_counts() != &expected_counts
        || usize::try_from(manifest.record_count()).ok() != Some(records.len())
        || logical_export_bytes(
            package,
            manifest.record_count(),
            manifest.canonical_source_bytes(),
            u64::MAX,
        )
        .ok()
            != Some(manifest.logical_export_bytes())
    {
        return Err(denial(DenialKind::IncompleteRecordClosure));
    }
    let mut actual_counts = [0_u32; WorthQueryPortablePackageRecordFamily::ALL.len()];
    for record in records {
        actual_counts[record.family().index()] = actual_counts[record.family().index()]
            .checked_add(1)
            .ok_or_else(|| denial(DenialKind::IncompleteRecordClosure))?;
    }
    if actual_counts != expected_counts || records != assemble_records(package, records.len()) {
        return Err(denial(DenialKind::IncompleteRecordClosure));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn verify_source_closure_for_test(
    package: &WorthQueryValidatedPortableDomainPackage,
    manifest: &WorthQueryPortablePackageManifest,
    records: &[WorthQueryPortablePackageRecord],
) -> Result<(), WorthQueryPortablePackageExportDenial> {
    validate_source_closure(package, manifest, records)
}

fn logical_export_bytes(
    package: &WorthQueryValidatedPortableDomainPackage,
    record_count: u32,
    canonical_source_bytes: u64,
    maximum_bytes: u64,
) -> Result<u64, WorthQueryPortablePackageExportDenial> {
    // Package canonical work already traverses every root and application-schema
    // payload, including the retained spine's source meaning. Domain operations
    // and artifact contracts enter that basis by digest, so add their complete
    // owner-defined canonical traversals instead of treating the digest as payload.
    let record_width = LOGICAL_RECORD_FAMILY_TAG_BYTES + LOGICAL_RECORD_CANONICAL_INDEX_BYTES;
    let framing_bytes = u64::from(record_count)
        .checked_mul(record_width)
        .and_then(|framing| canonical_source_bytes.checked_add(framing))
        .ok_or_else(|| denial(DenialKind::LogicalExportBytesExceeded))?;
    let logical_bytes = package
        .domain_operations()
        .iter()
        .try_fold(framing_bytes, |bytes, operation| {
            bytes.checked_add(operation.canonical_encoded_bytes())
        })
        .and_then(|bytes| {
            package
                .artifact_contracts()
                .iter()
                .try_fold(bytes, |bytes, contract| {
                    bytes.checked_add(contract.canonical_encoded_bytes())
                })
        })
        .ok_or_else(|| denial(DenialKind::LogicalExportBytesExceeded))?;
    if logical_bytes > maximum_bytes {
        return Err(denial(DenialKind::LogicalExportBytesExceeded));
    }
    Ok(logical_bytes)
}

const fn denial(kind: DenialKind) -> WorthQueryPortablePackageExportDenial {
    WorthQueryPortablePackageExportDenial::new(kind)
}
