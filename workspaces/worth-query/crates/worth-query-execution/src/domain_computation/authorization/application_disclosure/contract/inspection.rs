//! Only the actual cold binding may supply inspection metadata interpretation.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryInspectionMetadataFacet;
use worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract;

mod allocation;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use allocation::inspection_disclosure_backing_bytes;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelFailure,
};

/// Installation has already validated this exact immutable six-facet meaning.
/// The original reservation admits every new byte before its compiler copies.
pub(in crate::domain_computation) fn compile_inspection_disclosure_contract(
    inspection: &WorthQueryInstalledApplicationInspectionContract,
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryAdmittedApplicationDisclosureContract, CaptureStop<RetainedStorageDenial>> {
    run_retained_stage(execution, memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let bytes = inspection_disclosure_backing_bytes(inspection, context)?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
        }
        compile(inspection, context)
    })
}

fn compile(
    inspection: &WorthQueryInstalledApplicationInspectionContract,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<WorthQueryAdmittedApplicationDisclosureContract, MapKernelFailure<RetainedStorageDenial>>
{
    let contract = &inspection.definition().parts().metadata_disclosure;
    Ok(match contract.posture() {
        ApplicationQueryDisclosurePosture::Public => {
            WorthQueryAdmittedApplicationDisclosureContract::Public
        }
        ApplicationQueryDisclosurePosture::Governed => {
            let mut result_rules = BTreeMap::new();
            for rule in contract.rules() {
                checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                let slot = slot_for_rule(inspection, rule);
                let admitted = Arc::new(WorthQueryAdmittedApplicationDisclosureRule {
                    disclosure_value: rule.disclosure_value().clone(),
                    influence: rule.influence().clone(),
                    field: None,
                });
                result_rules.insert(slot.clone(), admitted);
            }
            WorthQueryAdmittedApplicationDisclosureContract::Governed {
                classification: contract.classification().to_owned(),
                capability_name: contract
                    .capability_name()
                    .expect("compiled governed inspection has an actual capability")
                    .to_owned(),
                capability_type: contract
                    .capability_type()
                    .expect("compiled governed inspection retains its capability type")
                    .to_owned(),
                result_rules,
                internal_rules: Vec::new(),
                internal_field_rules: BTreeMap::new(),
            }
        }
        ApplicationQueryDisclosurePosture::InstalledPolicyRequired => {
            unreachable!("inspection installation refuses implicit metadata policy")
        }
    })
}

fn slot_for_rule<'a>(
    inspection: &'a WorthQueryInstalledApplicationInspectionContract,
    rule: &ApplicationQueryDisclosureRule,
) -> &'a ApplicationQueryResultSlotKey {
    let ApplicationQueryDisclosureSelector::InspectionMetadata(metadata) = rule.selector() else {
        unreachable!("compiled inspection accepts only its closed metadata facets")
    };
    let index = ApplicationQueryInspectionMetadataFacet::ALL
        .iter()
        .position(|facet| *facet == metadata.facet())
        .expect("closed metadata facet has a cold slot");
    &inspection.metadata_slots()[index]
}
