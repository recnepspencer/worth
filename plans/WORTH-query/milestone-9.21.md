# Milestone 9.21: Governed Decision Attachments And Summaries

## Goal

Attach domain decisions and structural evidence to exact executions under
installed governance, then expose queryable incremental summaries without
promoting attachments or summaries to graph, operation, approval, or history
authority.

## Roadmap Placement

Milestones 9.19 and 9.20 produce exact access, footprint, path, partition, and
execution evidence. This milestone independently governs the derived decision-
evidence surface because classification, disclosure, retention, deletion, and
summary invalidation have different authority and lifecycle from execution; it
is not part of the Milestone 9.17 composite-branch phase sequence.

## Adversarial Constraint

A permissive outer attachment contains restricted nested content; a summary
omits a relevant update; an attachment is copied to another attempt or basis;
expired evidence survives through a derived summary; and a caller attempts to
use a decision label to approve, elevate, commit, undo, redo, recover, or
publish. Independent source-state and disclosure oracles must expose each lie.

## Product Decision Lock

1. Attachments bind exact schema, operation, attempt, basis, occurrence,
   artifact, verified footprint, and producing authority.
2. Domain packages own decision meaning and schema. Query owns generic
   governance, carriage, publication, and lifecycle.
3. Classification, redaction, retention, deletion, legal hold, purpose,
   capability, field disclosure, elevation, and mandatory review compose over
   every nested field.
4. Mandatory correctness facts remain in the canonical core. Optional rich
   explanation occupies policy-governed sidecars.
5. Summaries are derived projections with explicit dependencies, invalidation,
   freshness, omission, and rebuild posture.
6. Destroying all summaries must leave authoritative execution and attachment
   truth interpretable and permit complete rebuild where retention allows it.
7. Neither attachment nor summary can admit, approve, elevate, commit, recover,
   undo, redo, move a branch, or publish graph truth.
8. Search, ranking, aggregation, and cursor metadata obey the same protected-
   fact noninterference contract as result delivery.

## Destination Topology

```text
worth-query-installation/src/decision_evidence/
    attachment_schema.rs
    governance.rs
    summary_contract.rs

worth-query-execution/src/decision_evidence/
    attachment_production.rs
    governance_application.rs
    summary_maintenance.rs
    deletion.rs

worth-query-publication/src/decision_evidence/
    attachment.rs
    summary.rs
    omission.rs

worth-query-certification/src/reference_domains/
    bank_compliance/decision_evidence.rs
    research/decision_evidence.rs
```

## Phase Plan

### Phase 1: Installed Attachment Schemas

Establish typed schemas, exact execution/occurrence binding, canonical-core and
sidecar posture, and owner-only production authority.

### Phase 2: Governance And Publication

Compose classification, disclosure, redaction, retention, deletion, purpose,
capability, review, and omission semantics before publication.

### Phase 3: Incremental Summaries

Maintain queryable summaries with explicit dependencies, invalidation,
freshness, rebuild, disposal, and work evidence without making them authority.

### Phase 4: Bank/Research Adoption And Certification

Bank/compliance pressures protected audit and estate evidence; research
pressures occurrence identity and governed scientific decisions. Facade,
executable documentation, nested-disclosure, deletion, rebuild, stale-summary,
authority-spoofing, and residue courts close the milestone.

## Performance Contract

- Attachment production cost scales with declared attachment shape, not
  unrelated graph or history width.
- Summary maintenance scales with the relevant dependency delta plus its
  declared physical granule.
- Rich explanation, full reconstruction, and certification remain off the
  ordinary execution lane.
- Retained bytes, summary updates, rebuild work, redactions, and omitted
  sidecars are independently counted.

## Acceptance Evidence

Milestone 9.21 closes when restricted nested content cannot escape, summaries
update and rebuild exactly under relevant change, irrelevant change causes no
maintenance, deletion cannot be defeated by derived copies, copied evidence
cannot cross attempt/basis/occurrence boundaries, and no evidence surface opens
operational or history authority.

## Handoff

[Milestone 9.22](./milestone-9.22.md) may reuse eligible stages and subartifacts
only while preserving the exact execution, occurrence, dependency, provider,
and decision-governance meaning established here.
