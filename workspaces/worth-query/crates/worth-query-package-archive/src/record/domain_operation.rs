use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryDomainOperationIdentity, WorthQueryPortableDomainOperationParts,
    WorthQueryPortableDomainOperationRecord, WorthQueryPortableDomainOperationSemanticParts,
    WorthQueryPortableDomainOperationSemanticRecord, WorthQueryPortablePackageRecord,
};

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::limits::WorthQueryPackageArchiveLimits;

use super::decode_budget::RecordDecodeAttempt;
use super::encoding_budget::RecordPayloadEncodingWork;

mod artifact_reference;
mod canonical_query;
mod conditional_node;
mod input_contracts;
mod resource_contract;
mod semantic_contracts;
mod workflow;
mod workflow_value;

pub(super) fn payload_encoding_work(
    record: &WorthQueryPortableDomainOperationRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    let mut measure = BinaryEncodingMeasure::default();
    write_record(&mut measure, record)?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    record: &WorthQueryPortableDomainOperationRecord,
    output: &mut BinaryOutput,
) -> Result<(), Denial> {
    write_record(output, record)
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let identity = WorthQueryDomainOperationIdentity::new(input.text()?.to_owned(), input.u32()?);
    let parameters = input_contracts::decode_parameters(input, budget)?;
    let native_projection = input_contracts::decode_native_projection(input, budget)?;
    let canonical_query = canonical_query::decode_bundle(input, budget)?;
    let collection = input_contracts::decode_collection(input, budget)?;
    let required_capabilities = input_contracts::decode_capabilities(input, budget)?;
    let required_domains = input_contracts::decode_required_domains(input, budget)?;
    let workflow = workflow::decode_workflow(input, budget)?;
    let evidence = semantic_contracts::decode_evidence(input)?;
    let conditional_nodes = conditional_node::decode_nodes(input, budget)?;
    let graph_reads = semantic_contracts::decode_graph_reads(input, budget)?;
    let decision_facts = semantic_contracts::decode_decision_facts(input, budget)?;
    let touches = semantic_contracts::decode_touches(input, budget)?;
    let effects = semantic_contracts::decode_effects(input, budget)?;
    let invariants = semantic_contracts::decode_invariants(input, budget)?;
    let invariant_execution = semantic_contracts::decode_invariant_execution(input, budget)?;
    let replay = semantic_contracts::decode_replay(input)?;
    let (lineage, promotion, publication, projection_consumption) =
        semantic_contracts::decode_lifecycle(input)?;
    let (terminal, cost) = semantic_contracts::decode_terminal_cost(input, budget)?;
    let resources = resource_contract::decode_resource_contract(input, budget)?;
    let (support, lowering) = semantic_contracts::decode_support_lowering(input)?;
    let canonical_identity = input.text()?.to_owned();
    let semantics = WorthQueryPortableDomainOperationSemanticRecord::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableDomainOperationSemanticParts {
            parameters,
            native_projection,
            canonical_query,
            collection,
            required_capabilities,
            required_domains,
            workflow,
            evidence,
            conditional_nodes,
            graph_reads,
            decision_facts,
            touches,
            effects,
            invariants,
            invariant_execution,
            replay,
            lineage,
            promotion,
            publication,
            projection_consumption,
            terminal,
            cost,
            resources,
            support,
            lowering,
        },
    );
    Ok(WorthQueryPortablePackageRecord::DomainOperation(
        WorthQueryPortableDomainOperationRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableDomainOperationParts {
                identity,
                semantics,
                canonical_identity,
            },
        ),
    ))
}

fn write_record(
    output: &mut dyn BinaryEncodingSink,
    record: &WorthQueryPortableDomainOperationRecord,
) -> Result<(), Denial> {
    output.text(record.identity().name())?;
    output.u32(record.identity().version())?;
    let semantics = record.semantics();
    input_contracts::write_parameters(output, semantics.parameters())?;
    input_contracts::write_native_projection(output, semantics.native_projection())?;
    canonical_query::write_bundle(output, semantics.canonical_query())?;
    input_contracts::write_collection(output, semantics.collection())?;
    input_contracts::write_capabilities(output, semantics.required_capabilities())?;
    input_contracts::write_required_domains(output, semantics.required_domains())?;
    workflow::write_workflow(output, semantics.workflow())?;
    semantic_contracts::write_evidence(output, semantics.evidence())?;
    conditional_node::write_nodes(output, semantics.conditional_nodes())?;
    semantic_contracts::write_graph_reads(output, semantics.graph_reads())?;
    semantic_contracts::write_decision_facts(output, semantics.decision_facts())?;
    semantic_contracts::write_touches(output, semantics.touches())?;
    semantic_contracts::write_effects(output, semantics.effects())?;
    semantic_contracts::write_invariants(output, semantics.invariants())?;
    semantic_contracts::write_invariant_execution(output, semantics.invariant_execution())?;
    semantic_contracts::write_replay(output, semantics.replay())?;
    semantic_contracts::write_lifecycle(
        output,
        semantics.lineage(),
        semantics.promotion(),
        semantics.publication(),
        semantics.projection_consumption(),
    )?;
    semantic_contracts::write_terminal_cost(output, semantics.terminal(), semantics.cost())?;
    resource_contract::write_resource_contract(output, semantics.resources())?;
    semantic_contracts::write_support_lowering(output, semantics.support(), semantics.lowering())?;
    output.text(record.canonical_identity())
}
