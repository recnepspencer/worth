use sha2::{Digest, Sha256};

use super::{
    RequiredWorkflowAssessment, WorkflowAssessmentEvidenceMeaning, WorthQueryObservedSource,
    WorthQueryOutputDemandSettlement, WorthQueryWorkflowAssessmentPosture,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(super) fn output_content_identity(
    receipt: &WorthQueryApplicationCommitReceipt,
    source_identity: [u8; 32],
) -> String {
    let publication = receipt.committed_product_publication();
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-assessment-output-content:v1");
    digest.update(receipt.output_correspondence().workflow_content_identity());
    digest.update(source_identity);
    digest.update(receipt.installed_operation());
    digest.update(
        publication
            .product_branch()
            .owner_identity()
            .get()
            .to_le_bytes(),
    );
    digest.update(publication.product_branch().name().as_str().as_bytes());
    digest.update(publication.product_incarnation().ordinal().to_le_bytes());
    digest.update(publication.composite_commit().ordinal().to_le_bytes());
    digest.update(publication.relational_commit().commit_id.0.to_le_bytes());
    digest.update(publication.relational_commit().version_id.0.to_le_bytes());
    hex(digest.finalize().into())
}

pub(super) fn stable_output_identities(
    stable: &crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage,
    source_identity: [u8; 32],
) -> Option<(String, String)> {
    let partition = stable.source_partition_identity()?;
    let observation = stable.observation();
    let branch = observation.branch_identity();
    let (occurrence, generation, slot) = stable.exact_settlement().address();
    let mut publication = Sha256::new();
    publication.update(b"worth-query:workflow-assessment-stable-publication:v1");
    publication.update(observation.owner_identity().get().to_le_bytes());
    publication.update((branch.name().as_str().len() as u64).to_le_bytes());
    publication.update(branch.name().as_str().as_bytes());
    publication.update(occurrence.ordinal().to_le_bytes());
    publication.update(generation.to_le_bytes());
    publication.update((slot as u64).to_le_bytes());
    publication.update(observation.selected_commit().ordinal().to_le_bytes());
    let publication_bytes: [u8; 32] = publication.finalize().into();

    let mut output = Sha256::new();
    output.update(b"worth-query:workflow-assessment-stable-output-content:v1");
    output.update(stable.output_correspondence().workflow_content_identity());
    output.update(source_identity);
    output.update(partition);
    output.update(publication_bytes);
    Some((
        format!("stable:{}", hex(publication_bytes)),
        hex(output.finalize().into()),
    ))
}

pub(super) fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

pub(super) fn decode_hex_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&text[offset..offset + 2], 16).ok()?;
    }
    Some(bytes)
}

pub(super) fn evidence_meaning<Query>(
    required: &RequiredWorkflowAssessment,
    subject: worth_relational::facade::identity::EntityId,
    settlement: &WorthQueryOutputDemandSettlement,
    source: &WorthQueryObservedSource<Query>,
    posture: WorthQueryWorkflowAssessmentPosture,
    currentness_facts: std::sync::Arc<
        [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    >,
) -> Option<WorkflowAssessmentEvidenceMeaning> {
    let source_identity = hex(source.idempotency_identity().bytes());
    let (publication_identity, output_content_identity) =
        if let Some(receipt) = settlement.application_commit_receipt() {
            let publication = receipt.committed_product_publication();
            (
                format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}:{}",
                    publication.product_branch().owner_identity().get(),
                    publication.product_branch().name().as_str(),
                    publication.product_incarnation().ordinal(),
                    publication.product_generation().get(),
                    publication.composite_commit().ordinal(),
                    publication.publication_attempt().ordinal(),
                    publication.relational_commit().branch_id.0,
                    publication.relational_commit().commit_id.0,
                    publication.relational_commit().version_id.0,
                ),
                output_content_identity(receipt, source.idempotency_identity().bytes()),
            )
        } else {
            stable_output_identities(
                settlement.stable.as_ref()?,
                source.idempotency_identity().bytes(),
            )?
        };
    let passing = posture == WorthQueryWorkflowAssessmentPosture::Passing;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-assessment-evidence:v1");
    for value in [
        required.transition_identity(),
        settlement.producer_identity(),
        settlement.output_family_identity(),
        required.query(),
        required.parameter_type(),
        required.result_type(),
        required.binding(),
        required.proposal_identity(),
        required.coverage_identity(),
        required.program_revision(),
        &source_identity,
        &publication_identity,
        &output_content_identity,
    ] {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    digest.update(subject.partition_value_u64().to_le_bytes());
    digest.update(subject.local_slot_value().to_le_bytes());
    digest.update(u64::from(subject.generation_value()).to_le_bytes());
    digest.update([u8::from(passing)]);
    Some(WorkflowAssessmentEvidenceMeaning {
        identity: hex(digest.finalize().into()),
        producer: settlement.producer_identity().to_owned(),
        family: settlement.output_family_identity().to_owned(),
        query: required.query().to_owned(),
        parameter_type: required.parameter_type().to_owned(),
        result_type: required.result_type().to_owned(),
        binding: required.binding().to_owned(),
        subject,
        proposal_identity: required.proposal_identity().to_owned(),
        coverage_identity: required.coverage_identity().to_owned(),
        source_identity,
        passing,
        publication_identity,
        output_content_identity,
        program_revision: required.program_revision().to_owned(),
        retained_bytes: 0,
        currentness_facts,
    })
}
