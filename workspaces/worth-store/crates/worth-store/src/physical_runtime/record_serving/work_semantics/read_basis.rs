use worth_signal::facade::PartitionSubscription;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StorePhysicalBoundaryWitness,
};
use worth_store_physical_format::RecordArtifactFile;

use crate::physical_runtime::work::{
    PhysicalSignalAspectDeclaration, PhysicalSignalAspectRole, PhysicalWorkSemanticBasis,
    PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
};

use super::{contract, validated_value};

const ROOT_ASPECT_KEY: &str = "store.physical.record.root-read-basis";
const ARTIFACT_ASPECT_KEY: &str = "store.physical.record.artifact-read-basis";
const FRAME_ASPECT_KEY: &str = "store.physical.record.frame-read-basis";
const SCAN_ASPECT_KEY: &str = "store.physical.record.scan-read-basis";

const ROOT_PARTITION: &str = "store.physical.record.root";
const ARTIFACT_PARTITION: &str = "store.physical.record.artifact";
const FRAME_PARTITION: &str = "store.physical.record.frame";
const SCAN_PARTITION: &str = "store.physical.record.scan";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RecordReadPartition {
    Root,
    Artifact,
    Frame,
    Scan,
}

pub(super) struct InstalledRecordReadSemantics {
    pub(super) bases: RecordReadSemanticBases,
    pub(super) security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    pub(super) scheduler_security: worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    pub(super) declarations: [PhysicalSignalAspectDeclaration; 4],
}

#[derive(Clone, Debug)]
pub(super) struct RecordReadSemanticBases {
    root: PhysicalWorkSemanticBasis,
    artifact: PhysicalWorkSemanticBasis,
    frame: PhysicalWorkSemanticBasis,
    scan: PhysicalWorkSemanticBasis,
}

pub(super) fn install(witness: StorePhysicalBoundaryWitness) -> InstalledRecordReadSemantics {
    let root = read_semantics(
        ROOT_ASPECT_KEY,
        1_301,
        ROOT_PARTITION,
        "record-root-read-admitted",
        witness,
    );
    let (security, scheduler_security) =
        super::security_admission::read_security_admission(witness);
    let artifact = read_semantics(
        ARTIFACT_ASPECT_KEY,
        1_302,
        ARTIFACT_PARTITION,
        "record-artifact-read-admitted",
        witness,
    );
    let frame = read_semantics(
        FRAME_ASPECT_KEY,
        1_303,
        FRAME_PARTITION,
        "record-frame-read-admitted",
        witness,
    );
    let scan = read_semantics(
        SCAN_ASPECT_KEY,
        1_304,
        SCAN_PARTITION,
        "record-scan-read-admitted",
        witness,
    );
    InstalledRecordReadSemantics {
        bases: RecordReadSemanticBases {
            root: root.basis,
            artifact: artifact.basis,
            frame: frame.basis,
            scan: scan.basis,
        },
        security,
        scheduler_security,
        declarations: [
            root.declaration,
            artifact.declaration,
            frame.declaration,
            scan.declaration,
        ],
    }
}

impl RecordReadSemanticBases {
    pub(super) fn for_partition(
        &self,
        partition: RecordReadPartition,
    ) -> PhysicalWorkSemanticBasis {
        match partition {
            RecordReadPartition::Root => self.root.clone(),
            RecordReadPartition::Artifact => self.artifact.clone(),
            RecordReadPartition::Frame => self.frame.clone(),
            RecordReadPartition::Scan => self.scan.clone(),
        }
    }
}

impl RecordReadPartition {
    pub(in crate::physical_runtime) const fn for_range(artifact: RecordArtifactFile) -> Self {
        match artifact {
            RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector
            | RecordArtifactFile::RootSelectorCandidate { .. }
            | RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootManifest { .. }
            | RecordArtifactFile::RootRoutingBlock { .. }
            | RecordArtifactFile::ReleaseCustodyHeadBlock { .. } => Self::Root,
            RecordArtifactFile::Segment { .. } | RecordArtifactFile::ExtentArena { .. } => {
                Self::Frame
            }
            RecordArtifactFile::SegmentManifest { .. }
            | RecordArtifactFile::SegmentMembershipBlock { .. }
            | RecordArtifactFile::FreeSpaceManifest { .. }
            | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => Self::Artifact,
        }
    }

    pub(in crate::physical_runtime) const fn for_metadata(artifact: RecordArtifactFile) -> Self {
        match artifact {
            RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector
            | RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootManifest { .. }
            | RecordArtifactFile::RootRoutingBlock { .. }
            | RecordArtifactFile::ReleaseCustodyHeadBlock { .. } => Self::Root,
            _ => Self::Artifact,
        }
    }
}

struct ReadSemantics {
    basis: PhysicalWorkSemanticBasis,
    declaration: PhysicalSignalAspectDeclaration,
}

fn read_semantics(
    key: &'static str,
    contract_identity: u64,
    partition: &'static str,
    value: &'static str,
    witness: StorePhysicalBoundaryWitness,
) -> ReadSemantics {
    let (contract, identity, admission) = contract(key, contract_identity, witness);
    let value = validated_value(&contract, value);
    let state = match worth_foundational::aspects()
        .authoritative_state()
        .admit([value])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("built-in record aspect state must admit: {outcome:?}"),
    };
    let fact = StoreAspectBoundaryFact::from_admitted_state(
        identity,
        StoreAspectAuthorityInput::new(state, witness),
    )
    .expect("built-in record state contains exactly its declared identity");
    let basis = PhysicalWorkSemanticBasis::projection(fact, admission.clone())
        .expect("built-in read fact and contract are constructed together");
    let declaration =
        PhysicalSignalAspectDeclaration::new(admission, PhysicalSignalAspectRole::Dependency)
            .for_families(PhysicalWorkSignalFamilySet::only(
                PhysicalWorkSignalFamily::ReadFault,
            ))
            .with_partition(PartitionSubscription::whole_partition(partition));
    ReadSemantics { basis, declaration }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_read_semantics_are_distinct_and_exactly_partitioned() {
        let installed = install(super::super::security_admission::physical_witness());
        let expected = [
            (RecordReadPartition::Root, ROOT_ASPECT_KEY, ROOT_PARTITION),
            (
                RecordReadPartition::Artifact,
                ARTIFACT_ASPECT_KEY,
                ARTIFACT_PARTITION,
            ),
            (
                RecordReadPartition::Frame,
                FRAME_ASPECT_KEY,
                FRAME_PARTITION,
            ),
            (RecordReadPartition::Scan, SCAN_ASPECT_KEY, SCAN_PARTITION),
        ];
        for ((partition, aspect_key, partition_name), declaration) in
            expected.iter().zip(&installed.declarations)
        {
            let basis = installed.bases.for_partition(*partition);
            assert_eq!(basis.aspect_identity().aspect_key().as_str(), *aspect_key);
            let subscription = declaration
                .partition()
                .expect("read dependency is partitioned");
            assert_eq!(subscription.partition.0, *partition_name);
        }
        for (index, (partition, _, _)) in expected.iter().enumerate() {
            let basis = installed.bases.for_partition(*partition);
            for (other, _, _) in &expected[index + 1..] {
                assert_ne!(basis, installed.bases.for_partition(*other));
            }
        }
    }

    #[test]
    fn artifact_types_select_only_the_internal_read_partition_taxonomy() {
        assert_eq!(
            RecordReadPartition::for_range(RecordArtifactFile::RootManifest { generation: 3 }),
            RecordReadPartition::Root
        );
        assert_eq!(
            RecordReadPartition::for_range(RecordArtifactFile::SegmentManifest {
                segment: 7,
                generation: 3,
            }),
            RecordReadPartition::Artifact
        );
        assert_eq!(
            RecordReadPartition::for_range(RecordArtifactFile::ExtentArena { arena: 9 }),
            RecordReadPartition::Frame
        );
    }
}
