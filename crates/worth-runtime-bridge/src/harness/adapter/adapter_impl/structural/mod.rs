use crate::diagnostics::{
    BridgeCanonicalStructuralBranchComparisonRecord, BridgeCanonicalStructuralRemapRecord,
};
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::structural::{
    AdmittedStructuralComparisonContract, PlannedStructuralMatchPacketSet,
    PublishedBranchComparisonArtifact, PublishedStructuralRemapArtifact, ReducedStructuralMatchSet,
    StructuralIdentityDeclarationIdentity,
};

use super::*;

mod certification_bundle;
mod counter_snapshot;
mod diagnostics_digest;
mod execution_flows;
pub(in crate::harness::adapter::adapter_impl) mod terminal_report_export;

#[cfg(test)]
mod typed_certification_tests;

pub(super) enum StructuralHarnessTarget {
    RemapExact {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    RemapAmbiguous {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    RemapNoSafeMatch {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    RemapLineageDivergence {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    RemapIdentityConflict {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    RemapReplay {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    BranchCompare {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    BranchReplay {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
}

pub(super) enum StructuralHarnessExecution {
    Remap {
        contract: AdmittedStructuralComparisonContract,
        planned: PlannedStructuralMatchPacketSet,
        reduced: ReducedStructuralMatchSet,
        artifact: PublishedStructuralRemapArtifact,
        record: BridgeCanonicalStructuralRemapRecord,
    },
    RemapReplay {
        contract: AdmittedStructuralComparisonContract,
        planned: PlannedStructuralMatchPacketSet,
        reduced: ReducedStructuralMatchSet,
        artifact: PublishedStructuralRemapArtifact,
        record: BridgeCanonicalStructuralRemapRecord,
        replayed: PublishedStructuralRemapArtifact,
    },
    Branch {
        contract: AdmittedStructuralComparisonContract,
        planned: PlannedStructuralMatchPacketSet,
        reduced: ReducedStructuralMatchSet,
        artifact: PublishedBranchComparisonArtifact,
        record: BridgeCanonicalStructuralBranchComparisonRecord,
    },
    BranchReplay {
        contract: AdmittedStructuralComparisonContract,
        planned: PlannedStructuralMatchPacketSet,
        reduced: ReducedStructuralMatchSet,
        artifact: PublishedBranchComparisonArtifact,
        record: BridgeCanonicalStructuralBranchComparisonRecord,
        replayed: PublishedBranchComparisonArtifact,
    },
    Rejected {
        contract: AdmittedStructuralComparisonContract,
        planned: PlannedStructuralMatchPacketSet,
        reduced: ReducedStructuralMatchSet,
    },
}

impl StructuralHarnessExecution {
    fn summary(&self) -> certification_bundle::StructuralHarnessSummary {
        certification_bundle::StructuralHarnessSummary::from_execution(self)
    }

    fn certification_bundle(&self) -> certification_bundle::StructuralHarnessCertificationBundle {
        certification_bundle::StructuralHarnessCertificationBundle::from_execution(self)
    }
}

pub(super) fn execute_structural_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    target: StructuralHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    execution_flows::execute_structural_request(runtime_bridge, fixture, target, resource_request)
}

fn admitted_contract(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
) -> Result<AdmittedStructuralComparisonContract, BridgeHarnessError> {
    let declaration = fixture
        .structural_declarations()
        .iter()
        .find(|declaration| declaration.declaration_identity() == declaration_identity)
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge structural fixture does not declare `{}`",
                declaration_identity.as_str()
            ))
        })?;
    runtime_bridge
        .admit_structural_comparison(declaration)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge structural admission failed: {error}"))
        })
}

fn declaration_identity(
    contract: &AdmittedStructuralComparisonContract,
) -> &crate::structural::StructuralIdentityDeclarationIdentity {
    contract
        .validated_declaration()
        .declaration()
        .declaration_identity()
}
