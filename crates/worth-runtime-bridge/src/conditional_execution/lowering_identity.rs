use sha2::{Digest, Sha256};

use crate::correspondence::{BridgeCorrespondencePrecision, BridgeInstalledSemanticCorrespondence};

pub(super) fn installed_lowering_identity(
    bridge_runtime_key: u64,
    signal_branch_identity: &worth_signal::facade::branch::SignalBranchIdentity,
    signal_contract: &worth_signal::facade::InstalledSignalConditionalContract,
    declaration_identity: &str,
    correspondences: &[BridgeInstalledSemanticCorrespondence],
) -> String {
    let mut fields = vec![
        bridge_runtime_key.to_string(),
        signal_branch_identity.as_str().to_owned(),
        signal_contract.graph_instance_id().to_string(),
        signal_contract.node().index().to_string(),
        signal_contract.node().generation().to_string(),
        declaration_identity.to_string(),
    ];
    for correspondence in correspondences {
        fields.push(correspondence.dependency().canonical_registration_key());
        for target in correspondence.targets.as_slice() {
            fields.push(installed_target_basis(target));
        }
    }
    let canonical_basis = fields
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect::<String>();
    let digest = Sha256::digest(canonical_basis.as_bytes());
    format!("bridge-conditional-lowering:sha256:{digest:x}")
}

fn installed_target_basis(target: &crate::correspondence::InstalledCorrespondenceTarget) -> String {
    let precision = match target.precision {
        BridgeCorrespondencePrecision::Exact => "exact",
        BridgeCorrespondencePrecision::DeclaredWidening => "declared-widening",
    };
    let widening = match target.admitted_source_widening {
        None => "none",
        Some(crate::input::envelope::BridgeAspectChangeWideningCause::FieldToWholeAspect) => {
            "field-to-whole-aspect"
        }
        Some(crate::input::envelope::BridgeAspectChangeWideningCause::AspectToEntity) => {
            "aspect-to-entity"
        }
        Some(crate::input::envelope::BridgeAspectChangeWideningCause::SurfaceBroadening) => {
            "surface-broadening"
        }
        Some(
            crate::input::envelope::BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect,
        ) => "opaque-payload-to-whole-aspect",
    };
    [
        target.mapping_identity.as_ref(),
        &target.signal_graph_instance_id.to_string(),
        target.partition.0.as_str(),
        &target.node.index().to_string(),
        &target.node.generation().to_string(),
        &target.aspect.index().to_string(),
        precision,
        widening,
        &target.allocation_sources.join("|"),
    ]
    .into_iter()
    .map(|field| format!("{}:{field}", field.len()))
    .collect()
}
