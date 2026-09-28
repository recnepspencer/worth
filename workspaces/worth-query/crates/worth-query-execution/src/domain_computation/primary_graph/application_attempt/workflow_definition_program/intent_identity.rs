use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
    ApplicationWorkflowSpec,
};
use worth_relational::facade::identity::EntityId;

use super::{PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor};
use crate::basis::WorthQueryProductBranch;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.workflow-definition-publication-intent");
const RULE_VERSION: &str = "worth-query-workflow-definition-publication-intent-v2";
const RETIREMENT_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.workflow-definition-retirement-intent");
const RETIREMENT_RULE_VERSION: &str = "worth-query-workflow-definition-retirement-intent-v1";
const MAXIMUM_CANONICAL_BYTES: usize = 4 * 1_024 * 1_024;

pub(super) fn workflow_definition_intent_identity<Spec: ApplicationWorkflowSpec>(
    workflow_identity: &str,
    content_identity: &ApplicationWorkflowDefinitionContentIdentity,
    program_revision: &ApplicationProgramRevision,
    branch: WorthQueryProductBranch,
    predecessor: &WorkflowDefinitionExpectedPredecessor,
    assessment_bindings: &[(String, &'static str)],
    condition_bindings: &[(String, &'static str)],
    approval_bindings: &[worth_query_installation::facade::WorthQueryInstalledWorkflowApprovalBinding],
) -> Result<[u8; 32], ()> {
    let (predecessor_kind, predecessor_entity, predecessor_content) = match predecessor {
        WorkflowDefinitionExpectedPredecessor::Absent => ("absent", None, None),
        WorkflowDefinitionExpectedPredecessor::Published(predecessor) => (
            "published",
            Some(predecessor.entity_id()),
            Some(predecessor.content_identity()),
        ),
    };
    let mut entries = vec![
        entry(
            "spec",
            CanonicalBasisEntryKind::Identity,
            text(Spec::IDENTITY.as_str()),
        ),
        entry(
            "workflow",
            CanonicalBasisEntryKind::Identity,
            text(workflow_identity),
        ),
        entry(
            "content",
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*content_identity.as_bytes())),
        ),
        entry(
            "program-revision",
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*program_revision.as_bytes())),
        ),
        entry(
            "branch-occurrence",
            CanonicalBasisEntryKind::Identity,
            unsigned(branch.occurrence_ordinal()),
        ),
        entry(
            "predecessor-kind",
            CanonicalBasisEntryKind::Shape,
            text(predecessor_kind),
        ),
        entry(
            "predecessor-entity",
            CanonicalBasisEntryKind::Identity,
            predecessor_entity.map_or(CanonicalBasisValue::Null, entity),
        ),
        entry(
            "predecessor-content",
            CanonicalBasisEntryKind::Identity,
            predecessor_content.map_or(CanonicalBasisValue::Null, |identity| {
                CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*identity.as_bytes()))
            }),
        ),
    ];
    for (index, (path, binding)) in assessment_bindings.iter().enumerate() {
        entries.push(entry(
            format!("assessment.{index}.path"),
            CanonicalBasisEntryKind::Locator,
            text(path),
        ));
        entries.push(entry(
            format!("assessment.{index}.binding"),
            CanonicalBasisEntryKind::Identity,
            text(*binding),
        ));
    }
    for (index, (path, binding)) in condition_bindings.iter().enumerate() {
        entries.push(entry(
            format!("condition.{index}.path"),
            CanonicalBasisEntryKind::Locator,
            text(path),
        ));
        entries.push(entry(
            format!("condition.{index}.binding"),
            CanonicalBasisEntryKind::Identity,
            text(*binding),
        ));
    }
    for (index, binding) in approval_bindings.iter().enumerate() {
        entries.push(entry(
            format!("approval.{index}.path"),
            CanonicalBasisEntryKind::Locator,
            text(&binding.node_path),
        ));
        entries.push(entry(
            format!("approval.{index}.operation"),
            CanonicalBasisEntryKind::Identity,
            text(binding.operation),
        ));
        entries.push(entry(
            format!("approval.{index}.capability"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(
                binding.installed_capability_identity,
            )),
        ));
    }
    digest(RULE_VERSION, DOMAIN, entries)
}

/// Exact retirement intent: one published occurrence of one spec lineage,
/// retired under one installed program revision.
pub(super) fn workflow_definition_retirement_identity<Spec: ApplicationWorkflowSpec>(
    definition: &PublishedWorkflowDefinitionRef,
    program_revision: &ApplicationProgramRevision,
) -> Result<[u8; 32], ()> {
    let entries = vec![
        entry_in(
            RETIREMENT_DOMAIN,
            "spec",
            CanonicalBasisEntryKind::Identity,
            text(Spec::IDENTITY.as_str()),
        ),
        entry_in(
            RETIREMENT_DOMAIN,
            "definition",
            CanonicalBasisEntryKind::Identity,
            entity(definition.entity_id()),
        ),
        entry_in(
            RETIREMENT_DOMAIN,
            "content",
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(
                *definition.content_identity().as_bytes(),
            )),
        ),
        entry_in(
            RETIREMENT_DOMAIN,
            "program-revision",
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*program_revision.as_bytes())),
        ),
        entry_in(
            RETIREMENT_DOMAIN,
            "branch-occurrence",
            CanonicalBasisEntryKind::Identity,
            unsigned(definition.branch().occurrence_ordinal()),
        ),
    ];
    digest(RETIREMENT_RULE_VERSION, RETIREMENT_DOMAIN, entries)
}

fn digest(
    rule_version: &str,
    domain: CanonicalBasisDomain,
    entries: Vec<CanonicalBasisEntry>,
) -> Result<[u8; 32], ()> {
    let version = CanonicalizationRuleVersion::new(rule_version).ok_or(())?;
    let budget = CanonicalDigestWorkBudget::new(
        u32::try_from(entries.len()).map_err(|_| ())?,
        MAXIMUM_CANONICAL_BYTES,
    )
    .ok_or(())?;
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .map_err(|_| ())?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .map_err(|_| ())?;
    Ok(*canonicalization().digest().derive(ready).value().bytes())
}

fn entry(
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    entry_in(DOMAIN, locus, kind, value)
}

fn entry_in(
    domain: CanonicalBasisDomain,
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into().into()),
        kind,
        value,
    )
}

fn text(value: impl Into<String>) -> CanonicalBasisValue {
    CanonicalBasisValue::ExactText(value.into().into())
}

fn unsigned(value: u64) -> CanonicalBasisValue {
    CanonicalBasisValue::UnsignedInteger {
        width: CanonicalIntegerWidth::Bits64,
        value: u128::from(value),
    }
}

fn entity(value: EntityId) -> CanonicalBasisValue {
    CanonicalBasisValue::EntityRef {
        partition_id: value.partition_value(),
        local_slot: value.local_slot_value(),
        generation: value.generation_value(),
    }
}
