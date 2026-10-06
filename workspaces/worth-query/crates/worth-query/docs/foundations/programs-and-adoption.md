# Programs and Adoption

Audience: application authors and host integrators who ship more than one
revision of an application's meaning, and AI agents that generate that code.
Names in this document are reached through the audience facades
`worth-query-decl` and `worth-query-host`.

## In one paragraph

An application program is the validated meaning of an application: its rules,
outputs, actions, and features. Query keeps three identities apart. The
**program revision** is what the meaning is. **Support** is whether this host
installed that meaning and can run it. **Activation** is which revision one
branch occurrence actually runs. A host installs one initial program and may
roster further programs beside it. Each branch carries its own activation, and
a mutation never names its program: the host resolves the commit owner from
the program the request's branch carries. Moving a branch to another program
is **adoption**. For one branch, you compare the source and target, supply any
migration and workflow decisions the comparison demands, prepare, and publish
in one atomic step. For a set of branches, adoption is explicitly non-atomic:
each branch publishes separately, and a stopped set can be resumed, canceled,
or recovered. A host can stop supporting a program with
`retire_program_support` only after nothing still depends on it.

## Contents

1. [Terms](#terms)
2. [Where the names live](#where-the-names-live)
3. [The three identities](#the-three-identities)
4. [Author and validate a program](#author-and-validate-a-program)
5. [Roster programs on a host](#roster-programs-on-a-host)
6. [How a mutation reads the branch's program](#how-a-mutation-reads-the-branchs-program)
7. [Program commit denials](#program-commit-denials)
8. [Single-branch adoption at a glance](#single-branch-adoption-at-a-glance)
9. [Inspect and compare](#inspect-and-compare)
10. [Adoption requirements](#adoption-requirements)
11. [Migration](#migration)
12. [Workflow inventory and dispositions](#workflow-inventory-and-dispositions)
13. [Workflow disposition legality](#workflow-disposition-legality)
14. [Custody dispositions](#custody-dispositions)
15. [Prepare and publish](#prepare-and-publish)
16. [Recover an unpublished adoption](#recover-an-unpublished-adoption)
17. [Branch-set adoption](#branch-set-adoption)
18. [Resume, cancel, and recover a branch set](#resume-cancel-and-recover-a-branch-set)
19. [Retire program support](#retire-program-support)
20. [Denials](#denials)
21. [You can](#you-can)
22. [You cannot](#you-cannot)
23. [Misconceptions](#misconceptions)
24. [Worked example: an order application moves from revision 1 to revision 2](#worked-example-an-order-application-moves-from-revision-1-to-revision-2)
25. [Related documents](#related-documents)

## Terms

| Term | Meaning |
|---|---|
| Program | A type implementing `ApplicationProgramDefinition<Schema>`. It names the program's contributions, outputs, rules, identity (`IDENTITY`), and feature specs. |
| Validated program | `ValidatedApplicationProgram<Schema, Program>`, returned by `ApplicationProgramAuthoring::<Schema, Program>::begin().validated_program()`. |
| Program revision | `ApplicationProgramRevision`: a 32-byte canonical digest of validated program meaning. Two programs with the same meaning have the same revision. |
| Initial program | The program a host installs and activates at bootstrap. |
| Roster | The initial program plus every additional program the host supports, built with `WorthQueryApplicationProgramRoster`. |
| Support | A rostered program this host installed against its schema and has not retired. |
| Activation | The program revision one branch occurrence runs. |
| Adoption | Publishing a branch's move from its current (source) program to a rostered target program. |
| Requirements | `WorthQueryProgramAdoptionRequirements`: what one exact source-to-target move demands. |
| Migration | A target-owned mutation, prepared under the target program, that is committed atomically with the adoption. |
| Workflow inventory | `WorthQueryWorkflowAdoptionInventory`: the current workflow definitions and live workflow instances on the branch, each with the dispositions the host allows. |
| Custody disposition | What the host does with an in-flight operation continuation, external-effect recovery, or resource reservation when the program changes. The host derives it; you do not choose it. |
| Branch-set adoption | Adoption over an admitted, ordered set of branches, published one branch at a time. |
| Retirement | Removing a rostered program from ordinary host service with `retire_program_support`. |

## Where the names live

Paths are relative to `worth_query_host::facade` unless noted.

| Names | Module |
|---|---|
| `ApplicationProgramDefinition`, `ApplicationProgramAuthoring`, `ValidatedApplicationProgram`, `ApplicationProgramRevision`, `ApplicationSemanticChangeKind` | `declaration::application_program` (also through `worth-query-decl`) |
| `in_memory_rostered_program`, `WorthQueryApplicationProgramRoster`, `WorthQueryProgramApplicationRuntime`, `WorthQueryProgramSupportRetirementReceipt` | `application_installation` |
| `WorthQueryProgramAdoptionRequirements`, `WorthQueryProgramCustodyInventoryKind`, `WorthQueryProgramSupportRetirementDenial` | `domain` |
| Prepared, performed, unpublished, and branch-set adoption types; workflow inventory and disposition types; `WorthQueryProgramCustodyDispositionKind`; `WorthQueryProgramAdoptionCoverage` | `primary_graph` |
| `WorthQueryApplicationProgramsRequest`, `WorthQueryApplicationProgramAdoptionRequest`, `WorthQueryApplicationBranchSetProgramsRequest`, request denials | `application_entry` |

## The three identities

| Identity | Question it answers | Where it is decided | What proves it |
|---|---|---|---|
| Revision | What does this program mean? | Validation of the program definition | `ApplicationProgramRevision` from `ValidatedApplicationProgram::revision()` |
| Support | Can this host run it? | Rostered installation, until retirement | A `WorthQuerySupportedProgramHandle` from `supported_program::<P>()` |
| Activation | Does this branch occurrence run it? | Bootstrap, then adoption | The branch's activation record, read by the host at commit |

The rules that follow from the separation:

- Holding a revision grants nothing. A revision names meaning; it is not a
  receipt that any host supports it.
- Support grants nothing by itself. The supported-program handle's own
  documentation says activation still decides which rostered program an
  occurrence runs. A commit through a handle whose program is not the one the
  branch activated is refused before any effect.
- Activation is per branch. Adopting a program on one branch does not change
  any other branch. A branch forked before the adoption keeps the program it
  was created with.

## Author and validate a program

A program is a type implementing `ApplicationProgramDefinition<Schema>`.
Validation produces the revision:

```rust,ignore
use worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring;

let orders_v1 = ApplicationProgramAuthoring::<OrderSchema, OrdersV1>::begin()
    .validated_program()?;          // Result<_, ApplicationProgramValidationDenial>
let orders_v2 = ApplicationProgramAuthoring::<OrderSchema, OrdersV2>::begin()
    .validated_program()?;
let revision_2 = *orders_v2.revision();   // ApplicationProgramRevision is Copy
```

- `OrderSchema`, `OrdersV1`, and `OrdersV2` are your types.
- Both programs describe the same installed schema. A roster is installed
  against one schema.

## Roster programs on a host

`in_memory_rostered_program` validates and installs the whole roster before
the host serves anything:

```rust,ignore
use worth_query_host::facade::application_installation::{
    in_memory_rostered_program, WorthQueryApplicationProgramRoster,
};

let host = in_memory_rostered_program(
    orders_v1,                                              // initial program
    WorthQueryApplicationProgramRoster::new().support(orders_v2),
    declaration,
    configuration,
    limits,
    |bootstrap, installed_schema| Ok(()),                   // initial state
)?;                // Result<WorthQueryProgramApplicationRuntime<OrderSchema, OrdersV1>,
                   //        WorthQueryApplicationOpenDenial>
```

- The initial program is the one the host activates. Every other rostered
  program is reachable through `host.supported_program::<P>()`.
- Every rostered program is admitted against the same installed schema. The
  roster as a whole must leave no installed rule without a declaring owner.
- `supported_program::<P>()` returns `Option<WorthQuerySupportedProgramHandle>`.
  It is `None` when `P` was never rostered on this host. The handle offers
  `installed_program()` (whose `revision()` is the revision),
  `contains_action::<Binding>()`, and `runtime()`.
- `in_memory_rostered_program_from_checkpoint` restores the same roster from a
  Query-issued checkpoint, so a branch recorded under a supported program still
  runs it.
- A workflow vocabulary is installed per program. To keep a branch's workflows
  usable after it adopts another program, install the same spec against that
  program with `support_workflow_spec`. See [Workflows](workflows.md).

## How a mutation reads the branch's program

A mutation request never names a program revision. On the program lane:

```rust,ignore
let outcome = host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .mutate(PlaceOrderIntent { input })
    .without_source()
    .idempotency(&key)
    .execute_in_program(&host)?;
```

- `execute_in_program` resolves the commit owner from the program carried by
  the request's exact branch. No caller-supplied revision or activation receipt
  takes part in selection.
- If the branch's program removed this action, the host presents the initial
  installed owner only so the commit gate can publish its inactive-program
  denial. That presentation cannot commit the removed action or perform its
  external effects.
- A retried idempotency key replays its recorded outcome before any commit,
  even after the branch has adopted another program. Only a host that does not
  roster the branch's current program refuses the retry, at owner resolution,
  before the replay is consulted.
- `execute_capability_in_program` is the capability counterpart with the same
  rules.
- A binding that needs a program and is sent through the ordinary `execute`
  lane is refused before any effect with
  `WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired`. On
  the program lane that denial means neither the branch's program nor the
  initial installation supplies the action.
- `WorthQueryApplicationRequestMutationDenial::ProgramSelection(..)` means the
  branch's program could not be inspected, so no installed owner was chosen.
- Neither request-time refusal claims the idempotency key.

## Program commit denials

These are `WorthQueryApplicationCommitDenialKind` variants. The program ones are
raised at the `ProposalBinding` stage, before any effect.

| Kind | Meaning |
|---|---|
| `ProgramNotActiveOnOccurrence { active }` | The presented program is not the one this occurrence runs. `active` is the revision it does run. |
| `ProgramSupportRetired` | The program's support on this host is retired or is being retired. |
| `ProgramActivationUnresolved` | The host cannot attribute the branch's activation to a program it admitted. |
| `ApplicationProgramRequired` | The action needs an application program and none supplied it. |

## Single-branch adoption at a glance

| Step | Call | Returns |
|---|---|---|
| 1 | `request.on_branch(b).programs()` | `WorthQueryApplicationProgramsRequest` |
| 2 | `.inspect()` (optional) | `WorthQuerySelectedProgramInspection` |
| 3 | `.compare(&target)` | `WorthQueryProgramAdoptionRequirements` |
| 4 | `.adopt(&requirements)` | `WorthQueryApplicationProgramAdoptionRequest` |
| 5 | `.migration(prepared)` if the requirements demand an assessment | same request |
| 6 | `.workflow_inventory(max)`, then `.workflow(dispositions)` if the branch holds workflow facts | same request |
| 7 | `.prepare(max_selection_work)` | `WorthQueryPreparedBranchAdoption` |
| 8 | `.publish()` | `WorthQueryBranchAdoptionPublicationOutcome` |
| 9 | `programs().recover(recovery)` if step 8 was `ProductUnpublished` | `WorthQueryBranchAdoptionRecoveryOutcome` |

Nothing before `publish` changes the branch.

## Inspect and compare

- `inspect()` reports the rostered program this exact branch occurrence carries,
  as `WorthQuerySelectedProgramInspection` with `revision()`. The report is
  descriptive. It is not adoption preparation or publication authority. Its
  denial is `WorthQueryApplicationProgramInspectionDenial` (`ProductSelection`
  or `Inspection`).
- `compare(&target)` derives the requirements for moving this branch from its
  current program to `target`. Its denial is
  `WorthQueryApplicationProgramAdoptionPreparationDenial` (`ProductSelection` or
  `Adoption`).

## Adoption requirements

`WorthQueryProgramAdoptionRequirements` describes one exact source-to-target
move.

| Method | What it reports |
|---|---|
| `source()`, `target()` | The two revisions. |
| `semantic_diff()` | The semantic changes. Each change has a kind: `Added`, `Removed`, or `Changed` (`ApplicationSemanticChangeKind`). |
| `validation_scopes()`, `requires_existing_state_validation()` | Existing state the target's rules must re-check. |
| `added_rules()` | Rules the target adds. |
| `migration_assessment_requirements()`, `requires_migration_assessment()` | Changes that need a migration before the target can accept the branch. |
| `custody_inventory_requirements()`, `requires_custody_inventory()` | In-flight custody the move must account for. |
| `changed_workflow_dependencies()` | Workflow dependencies the move changes. |
| `semantically_equivalent()` | A positive authored equivalence. It does not claim that changed target rules accepted live state. |

`adopt(&requirements)` binds the adoption to these exact requirements.
Preparation refuses requirements the branch no longer matches, with
`RequirementsChanged`. Compare again and retry.

## Migration

A semantic change requires a migration assessment when it changes or removes
a feature, port, connection, or output. Added items and rule changes do not;
an added rule instead adds a validation scope, and existing state that
violates it is refused at preparation with `TargetRuleRejected`.

Requirements are derived from the two programs and the installed schema, not
from a branch's data. Every branch moving between the same two revisions gets
the same migration assessment requirements.

When `requires_migration_assessment()` is true, preparation without a
migration is refused with `MigrationAssessmentRequired(requirement)`. A
migration is an ordinary mutation executed as a candidate under the target
program:

```rust,ignore
use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMigrationPreparationOutcome as Outcome;

let migration = match runtime
    .request(&principal, &scope)
    .on_branch(branch)
    .mutate(BackfillOrderStatusIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_program_migration(&revision_2)?
{
    Outcome::Prepared(prepared) => prepared,     // WorthQueryPreparedProgramMigration
    Outcome::DomainDenied(denial) => return Err(denial.into()),
    Outcome::Cancelled | Outcome::DeadlineExceeded => return Err(retry_later()),
};
```

- `prepare_program_migration` commits nothing. It registers no idempotency or
  outbox record. The migration commits only as part of the adoption.
- A binding that requires workflow authority is refused with
  `WorthQueryApplicationProgramMigrationPreparationDenial::Request(RequiresWorkflowTransition)`.
- The prepared migration offers `target()` and `description().effect_count()`.
- Pass it with `.migration(prepared)` before `prepare`. Preparation refuses a
  migration prepared for another target (`MigrationTargetMismatch`) or against
  branch state that has since moved (`MigrationSourceChanged`).

## Workflow inventory and dispositions

A branch holding workflow facts needs one explicit decision per current
workflow definition and per live workflow instance. Preparing without them is
refused with `WorkflowDispositionRequired { inventory }`, which carries the
inventory. You can also read it first:

```rust,ignore
let adoption = programs.adopt(&requirements);
let inventory = adoption.workflow_inventory(1_024)?;   // WorthQueryWorkflowAdoptionInventory
```

The inventory:

| Method | Meaning |
|---|---|
| `definitions()`, `definition(entity)` | Current definitions (`WorthQueryWorkflowDefinitionOccurrence`). |
| `instances()`, `instance(entity)` | Live instances (`WorthQueryWorkflowInstanceOccurrence`). Historical instances copied into a fork are not listed. |
| `is_empty()`, `digest()`, `work_units()`, `source()`, `target()` | Shape and identity of the inventory. |
| `dispositions()` | An empty `WorthQueryWorkflowDispositions` bound to this inventory's digest. |
| `carry_compatible()` | Chooses `Carry` for everything, refusing at the first occurrence that cannot be carried. |

Each occurrence reports:

- `compatibility()`: `WorthQueryWorkflowCompatibility::Compatible`, or
  `Incompatible(WorthQueryWorkflowIncompatibility)` with one of
  `VocabularyUnsupported { spec }`, `NodeUncovered { node_path }`, or
  `DependencyChanged { node_path, name }`.
- `legal_dispositions()`: the only dispositions the builder accepts.
- For instances, also `custody()` (`WorthQueryWorkflowInstanceCustody`) and
  `requires_migration()`.

Build the decisions by chaining. Each call consumes the builder and returns
`Result<WorthQueryWorkflowDispositions, WorthQueryWorkflowDispositionDenial>`:

```rust,ignore
use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowDefinitionDisposition as Definition,
    WorthQueryWorkflowInstanceDisposition as Instance,
};

let choices = inventory
    .dispositions()
    .definition(&inventory.definitions()[0], Definition::Carry)?
    .instance(&inventory.instances()[0], Instance::Cancel)?;
let prepared = adoption.workflow(choices).prepare(1_024)?;
```

- A definition disposition is `Carry` or `Retire`.
- An instance disposition is `Carry` or `Cancel`. After adoption cancels an
  instance, requests on it are refused with the attempt denial
  `WorkflowInstanceCancelled`, the same one an explicit cancellation produces.
- If owner truth moves after you decide, preparation is refused with
  `WorkflowInventoryChanged { inventory }`. Decide the new inventory and prepare
  again.

## Workflow disposition legality

The host computes legality. You cannot widen it.

| Occurrence | Compatible with target | Instance custody | Legal dispositions |
|---|---|---|---|
| Definition | yes | not applicable | `Carry`, `Retire` |
| Definition | no | not applicable | `Retire` |
| Instance | yes | `Unperformed` | `Carry`, `Cancel` |
| Instance | yes | `Performed` | `Carry` |
| Instance | no | `Unperformed` | `Cancel` |
| Instance | no | `Performed` | none: migration required |
| Instance | any | `ApprovalOutstanding { approval_node_path }` | none |
| Instance | any | `OperationInOwnerCustody` | none |

- An instance that has performed an effect cannot be canceled by adoption.
- Every row with no legal disposition is refused the same way, with
  `MigrationRequired { instance }` (`requires_migration()` is true exactly when
  `legal_dispositions()` is empty). That covers an incompatible performed
  instance, an instance with an approval whose operation has not settled
  (`ApprovalOutstanding`), and an instance with an operation in owner custody
  (`OperationInOwnerCustody`).
- To clear it, settle or recover the instance under the source program, or
  migrate it with `prepare_workflow_instance_migration` (see
  [Workflows](workflows.md)), then take a fresh inventory.

## Custody dispositions

When `requires_custody_inventory()` is true, the move crosses in-flight
custody. The inventory kinds (`WorthQueryProgramCustodyInventoryKind`) are
`OperationContinuation`, `ExternalEffectRecovery`, and `ResourceCustody`.

The host derives one disposition for each requirement. Read them after
preparation with `prepared.custody().dispositions()`; each item offers
`requirement()` and `kind()`.

| `WorthQueryProgramCustodyDispositionKind` | Meaning |
|---|---|
| `FreshCurrentAdmission` | The operation still exists with changed meaning. A continuation grants no authority and must enter the target program through fresh admission. |
| `RetireUneffectedContinuation` | The operation no longer exists. An uneffected continuation closes under its source meaning. |
| `RetainExactEffectRecovery` | A performed effect keeps the exact occurrence and recovery authority recorded when it committed. |
| `RetainExactSourceReservation` | An already-admitted source reservation is kept instead of borrowing a changed target ceiling. |

If the host has no disposition for a requirement, preparation is refused with
`CustodyDispositionUnsupported(requirement)`.

## Prepare and publish

`prepare(max_selection_work)` validates the branch's existing state against the
target's rules within a work bound, stages the migration if any, and reserves
the publication. It returns `WorthQueryPreparedBranchAdoption`:

| Method | Meaning |
|---|---|
| `source()`, `target()`, `requirements()` | What is being adopted. |
| `selected_entity_count()`, `selection_work_units()` | How much state was validated. |
| `migration()`, `custody()` | The staged migration and derived custody dispositions. |
| `publish()` | Publishes the move. |

`publish()` returns `WorthQueryBranchAdoptionPublicationOutcome`:

| Outcome | Meaning | Next step |
|---|---|---|
| `Performed(WorthQueryPerformedBranchAdoption)` | The branch now runs the target. `target()`, `migration()`, `custody()` describe it. | None. |
| `NoEffect(NoEffectCompositePublication)` | Nothing was published. The branch still runs the source. | Prepare again if you still want the move. |
| `ProductUnpublished(WorthQueryUnpublishedBranchAdoption)` | Owner effects started but the product publication did not complete. | Recover. |

A work bound that is too small is refused with
`SelectionLimitExceeded { maximum_work_units, consumed_work_units }`.

## Recover an unpublished adoption

`WorthQueryUnpublishedBranchAdoption` offers `source()`, `target()`,
`selected_entity_count()`, `migration()`, `custody()`, `cause()`,
`expected_product()`, `next_actions()`, `relational_requires_settlement()`, and
`into_recovery()`.

```rust,ignore
let recovery = unpublished.into_recovery();       // WorthQueryBranchAdoptionRecovery
let outcome = runtime
    .request(&principal, &scope)
    .on_branch(branch)
    .programs()
    .recover(recovery)?;                          // WorthQueryBranchAdoptionRecoveryOutcome
```

| `WorthQueryBranchAdoptionRecoveryOutcome` | Meaning |
|---|---|
| `Performed { adoption, cleanup }` | Recovery completed the adoption. |
| `NoEffect { no_effect, recovery }` | This attempt published nothing. Custody is returned in `recovery`. |
| `ProductUnpublished { next, prior_cleanup }` | Still unpublished. Continue with `next`. |

- `recover` accepts only the exact custody an unpublished adoption returned. A
  raw runtime-world handle or receipt does not compile as its argument.
- On failure, `WorthQueryApplicationProgramAdoptionRecoveryFailure` hands the
  custody back through `into_recovery()`.

## Branch-set adoption

Branch-set adoption moves an exact, bounded set of live branches to one target.
It is explicitly **non-atomic**: each branch publishes on its own.

1. Admit the set. `runtime.branches().program_adoption_coverage(&branches, maximum_targets)`
   returns `WorthQueryProgramAdoptionCoverage`. It admits exactly the branches
   named. Branches created later are not added.
2. Order it. `request.on_branches(coverage, &ordered_targets)` fixes the
   publication order. It is refused if the order does not match the coverage.
3. Prepare. `.programs().adopt(&target).prepare(maximum_selection_work_per_branch)`
   returns `WorthQueryPreparedBranchSetAdoption`. Every branch is preflighted
   before any branch publishes.
4. Advance. Each `advance()` publishes the next branch and returns its
   `WorthQueryBranchSetAdoptionProgress`: `Performed { branch, adoption }`,
   `NoEffect { branch, no_effect }`, or `ProductUnpublished { branch, adoption }`.
   It returns `Ok(None)` when nothing is pending.
5. Close. `close()` returns `WorthQueryClosedBranchSetAdoption` when no branch
   is pending and none needs resolution. Otherwise it returns the adoption
   unchanged; `close_denial()` says why (`Pending { remaining }` or
   `ResolutionRequired { branch }`).

```rust,ignore
use std::num::NonZeroUsize;

let coverage = runtime
    .branches()
    .program_adoption_coverage(&branches, NonZeroUsize::new(16).unwrap())?;
let mut adoption = runtime
    .request(&principal, &scope)
    .on_branches(coverage, &branches)?
    .programs()
    .adopt(&revision_2)
    .prepare(1_024)?;
while let Some(progress) = adoption.advance()? {
    record(progress);
}
let closed = adoption.close().map_err(|open| open.close_denial())?;
```

Branch-set preparation passes no migration and no workflow dispositions. A
branch that needs either is refused as
`WorthQueryBranchSetAdoptionPreparationDenial::Adoption { branch, denial }`,
where `denial` is, for example, `MigrationAssessmentRequired` or
`WorkflowDispositionRequired`. Adopt such branches one at a time with the
single-branch flow first, then adopt the rest as a set.

## Resume, cancel, and recover a branch set

After a `NoEffect` or `ProductUnpublished` step, `advance()` is refused with
`ResolutionRequired { branch }` until you resolve the stop.

| Situation | Call | Result |
|---|---|---|
| Stopped on `NoEffect`, want to continue | `begin_resume()` | `WorthQueryStoppedBranchSetAdoption`. It releases the prepared suffix and keeps exact progress. |
| Continue the stopped set | Issue fresh coverage for `stopped.remaining_branches()`, then `request.on_branches(coverage, stopped.remaining_branches())?.programs().resume(stopped, max)` | A new `WorthQueryPreparedBranchSetAdoption`. Every remaining branch is preflighted again. |
| Give up on a stopped set | `stopped.cancel()` | `WorthQueryBranchSetAdoptionCancellation`. |
| Give up before finishing | `cancel()` | Cancels branches whose owner effects have not started. Completed branches stay completed. A `NoEffect` stop is resolved by cancellation. Refused while a `ProductUnpublished` branch holds custody. |
| Stopped on `ProductUnpublished` | `begin_recovery()`, then `request.recover_branch_set_adoption(recovery)` | Continues exact unpublished custody with the performed prefix and untouched suffix kept together. |
| Abandon unpublished custody | `request.release_branch_set_adoption_recovery(recovery, minimum_age_ticks)` | Releases the custody and cancels only the untouched suffix. The performed prefix remains as terminal evidence. |

- `resume` requires coverage that matches the stopped operation's exact
  remaining order. Otherwise `WorthQueryBranchSetAdoptionResumeFailure` returns
  the stopped adoption through `into_adoption()`; its `denial()` is one of
  `CoverageMismatch`, `RetentionAllocationRejected`, `Preparation(..)`, or
  `WorkAccountingOverflow`.
- `WorthQueryBranchSetAdoptionCancellation` offers `cancelled_branch_count()`,
  `progress()`, and `total_selection_work_units()`.
- `WorthQueryBranchSetAdoptionRecoveryOutcome` is `Performed { adoption, cleanup }`
  (continue advancing `adoption`), `NoEffect { no_effect, recovery }`, or
  `ProductUnpublished { recovery, prior_cleanup }`.

## Retire program support

```rust,ignore
let receipt = host.retire_program_support(&revision_1)?;
// WorthQueryProgramSupportRetirementReceipt, with inventory()
```

Retirement removes one rostered program from ordinary host service. It succeeds
only after every live branch, retained interpretation, and mandatory
adoption-recovery obligation that depends on the revision has been inventoried
and released.

| `WorthQueryProgramSupportRetirementDenial` | Meaning |
|---|---|
| `UnrosteredProgram { revision }` | This host never rostered the revision. |
| `AlreadyRetired { revision }` | Already retired. |
| `RetirementInProgress { revision }` | Another retirement of this revision is running. |
| `CurrentBranches(inventory)` | Live branches still run the revision. Adopt them away first. |
| `RetainedInterpretation(inventory)` | Retained state still needs the revision to be interpreted. |
| `MandatoryCustody(inventory)` | An adoption-recovery obligation still needs the revision. |
| `InventoryUnavailable(partial)` | The host could not complete the inventory. |

While a retirement runs, adoption to that revision is refused with
`ProgramSupportRetirementInProgress`, and commits under it get
`ProgramSupportRetired`.

## Denials

### Single-branch adoption preparation

`WorthQueryBranchAdoptionPreparationDenial`, reached as
`WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(..)`:

| Group | Variants | What to do |
|---|---|---|
| Support and activation | `ProgramSupportUnavailable`, `ProgramActivationUnavailable`, `ProgramActivationUnreadable`, `ProgramActivationUnrostered`, `ProgramSupportRetirementInProgress` | Check the roster and retirement state. The source or target is not usable on this host. |
| Requirements | `Requirements(WorthQueryProgramAdoptionRequirementsDenial)`, `RequirementsChanged` | Compare again. |
| Migration and custody | `MigrationAssessmentRequired(..)`, `MigrationTargetMismatch`, `MigrationSourceChanged`, `CustodyDispositionUnsupported(..)` | Supply or re-prepare the migration; resolve unsupported custody. |
| Workflow | `WorkflowDispositionRequired { inventory }`, `WorkflowInventoryChanged { inventory }`, `WorkflowDispositionRejected(..)`, `WorkflowInventoryUnreadable { .. }`, `WorkflowInventoryRelationUnreadable { .. }` | Decide the carried inventory and prepare again. |
| Scope and basis | `UnknownEntityScope { .. }`, `RelationScopeUnsupported { .. }`, `SelectionLimitExceeded { .. }`, `BranchBasisUnavailable(..)` | Raise the work bound or re-select the branch. |
| Transaction and publication | `TransactionAdmission(..)`, `TransactionStaging(..)`, `TargetRuleRejected { .. }`, `RelationalPreparation(..)`, `WorldPreparation(..)`, `ProductActivation(..)` | `TargetRuleRejected` means existing state violates a target rule: migrate it. The rest are storage or publication refusals. |

`ProductActivation` carries `WorthQueryBranchAdoptionActivationDenial`:
`CapacityExhausted`, `AllocationRejected`, `UnknownProductBranch`,
`RegistryUnavailable`, `GateUnavailable`, `PublicationInProgress`, or
`ProgramSupportUnavailable`.

### Workflow dispositions

| `WorthQueryWorkflowDispositionDenial` | Meaning |
|---|---|
| `IllegalDefinitionDisposition { definition, disposition }` | Not in the definition's `legal_dispositions()`. |
| `IllegalInstanceDisposition { .. }` | Not in the instance's `legal_dispositions()`. |
| `MigrationRequired { instance }` | The instance has no legal disposition: incompatible and performed, `ApprovalOutstanding`, or `OperationInOwnerCustody`. Settle, recover, or migrate it, then re-inventory. |
| `UnknownDefinition { definition }`, `UnknownInstance { instance }` | The occurrence is not in this inventory. |
| `DefinitionUndecided { definition }`, `InstanceUndecided { instance }` | A required decision is missing. |

### Branch sets

| Denial | Variants |
|---|---|
| `WorthQueryProgramAdoptionCoverageDenial` | `EmptyCoverage`, `TargetLimitExceeded { maximum, presented }`, `AllocationRejected`, `DuplicateTarget { branch }`, `ForeignOrRetiredTarget { branch }`, `RegistryUnavailable`, `ForeignApplication`, `OrderedTargetCountMismatch { covered, ordered }`, `OrderedTargetMismatch` |
| `WorthQueryBranchSetAdoptionPreparationDenial` | `RetentionAllocationRejected`, `ProductSelection { branch, denial }`, `Adoption { branch, denial }`, `WorkAccountingOverflow` |
| `WorthQueryBranchSetAdoptionAdvanceDenial` | `ResolutionRequired { branch }` |
| `WorthQueryBranchSetAdoptionCloseDenial` | `Pending { remaining }`, `ResolutionRequired { branch }` |

## You can

- Roster several programs on one host and run different branches on different
  programs at the same time.
- Read which program a branch runs with `programs().inspect()`.
- Compare a branch's program with any rostered target before deciding.
- Commit a target-owned migration atomically with the adoption.
- Carry, retire, or cancel workflow facts within the legal dispositions.
- Retry a mutation's idempotency key after its branch adopted another program
  and get the recorded outcome.
- Adopt many branches as a set, stop, resume with fresh coverage, cancel the
  untouched suffix, or recover unpublished custody.
- Retire a program once no branch, retained interpretation, or recovery
  obligation needs it.

## You cannot

- Name a program revision on a mutation to choose its owner.
- Use a revision, a supported-program handle, or an inspection report as proof
  that a branch runs that program.
- Adopt a program the host did not roster.
- Choose a custody disposition. The host derives it.
- Cancel a workflow instance that has already performed an effect, or give any
  disposition to an instance with an outstanding approval or an operation in
  owner custody.
- Pass migrations or workflow dispositions through branch-set adoption.
- Expect branch-set adoption to be all-or-nothing, or to include branches
  created after coverage was issued.
- Retire a program that a live branch still runs.

## Misconceptions

| Misconception | Correct statement |
|---|---|
| "The host runs one program." | The host rosters many; each branch occurrence activates one. |
| "Rostering a program switches branches to it." | Rostering adds support only. Branches move only by adoption. |
| "Adoption changes every branch." | Adoption changes the selected branch. Other branches, including earlier forks, keep their programs. |
| "`semantically_equivalent()` means existing data is valid under the target." | It is a positive authored equivalence. It does not claim changed target rules accepted live state. |
| "`prepare_program_migration` commits the migration." | It commits nothing and registers no idempotency record. The migration commits only with the adoption. |
| "Branch-set adoption is a transaction." | It is non-atomic by design. Performed branches stay performed if a later branch stops. |
| "Only branches with affected data need a migration." | Migration assessment requirements come from the two programs, so every branch making the same move needs one. Existing data matters for rule validation, not for whether an assessment is required. |
| "`inspect()` authorizes adoption." | It is descriptive only. `compare` and `adopt` are the authority path. |
| "A retried key after adoption re-runs under the new program." | It replays the recorded outcome before any commit. |

## Worked example: an order application moves from revision 1 to revision 2

**Scenario.** An order application runs revision 1 (`OrdersV1`). Revision 2
(`OrdersV2`) changes the order-entry feature so each order carries a
fulfillment status, and adds a rule that every order has one. The changed
feature requires a migration assessment, and the added rule requires existing
orders to be validated. The main branch also holds an approval workflow whose
nodes do not depend on the changed feature, so its definition stays
compatible, and one live instance has not performed anything yet. The team
moves the main branch, then each preview branch, then retires revision 1.

**1. Install both revisions.** The host activates revision 1 and rosters
revision 2.

```rust,ignore
let v1 = ApplicationProgramAuthoring::<OrderSchema, OrdersV1>::begin().validated_program()?;
let v2 = ApplicationProgramAuthoring::<OrderSchema, OrdersV2>::begin().validated_program()?;
let (revision_1, revision_2) = (*v1.revision(), *v2.revision());
let host = in_memory_rostered_program(
    v1,
    WorthQueryApplicationProgramRoster::new().support(v2),
    declaration, configuration, limits,
    |bootstrap, installed_schema| seed_orders(bootstrap, installed_schema),
)?;
let runtime = host.runtime();
```

**2. Compare.**

```rust,ignore
let programs = runtime.request(&principal, &scope).on_branch(main).programs();
let requirements = programs.compare(&revision_2)?;
assert!(requirements.requires_existing_state_validation());
assert!(requirements.requires_migration_assessment());
```

**3. Prepare the migration** under revision 2, as a mutation that sets a
status on every existing order.

```rust,ignore
let migration = match runtime
    .request(&principal, &scope)
    .on_branch(main)
    .mutate(BackfillOrderStatusIntent { input })
    .without_source()
    .idempotency(&backfill_key)
    .prepare_program_migration(&revision_2)?
{
    WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
    other => return Err(migration_failed(other)),
};
```

**4. Decide workflow facts.** The definition is compatible, so `Carry` is legal.
The instance is compatible and `Unperformed`, so `Carry` is legal too.

```rust,ignore
let adoption = programs.adopt(&requirements).migration(migration);
let inventory = adoption.workflow_inventory(1_024)?;
let choices = inventory.carry_compatible()?;
```

**5. Prepare and publish.**

```rust,ignore
let prepared = adoption.workflow(choices).prepare(4_096)?;
match prepared.publish() {
    WorthQueryBranchAdoptionPublicationOutcome::Performed(done) => assert_eq!(done.target(), &revision_2),
    WorthQueryBranchAdoptionPublicationOutcome::NoEffect(_) => retry_from_compare(),
    WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
        let recovery = unpublished.into_recovery();
        runtime.request(&principal, &scope).on_branch(main).programs().recover(recovery)?;
    }
}
```

After this, mutations on `main` resolve to revision 2. Mutations on the
preview branches still resolve to revision 1.

**6. Move each preview branch.** Requirements come from the programs, not the
data, so every preview branch moving from revision 1 to revision 2 needs its
own migration. Branch-set preparation would refuse each one with
`Adoption { branch, denial: MigrationAssessmentRequired(..) }`. Repeat steps 2
to 5 for each preview branch; skip step 4 on a branch whose inventory
`is_empty()`. Branch-set adoption fits a move whose requirements need no
migration, across branches that hold no workflow facts.

**7. Retire revision 1.**

```rust,ignore
let receipt = host.retire_program_support(&revision_1)?;
```

If any branch still runs revision 1, this is refused with
`CurrentBranches(inventory)`, and the inventory names what remains.

**Reference.** The Bank reference world (`workspaces/worth-query-bank-world`)
exercises rostered programs and workflows in a complete application. The
certification suite in `worth-query-certification` covers adoption, migration,
workflow custody, branch sets, and support retirement.

## Related documents

- [Query AI README](../AI_README.md)
- [Ordinary application front door](ordinary-application-front-door.md)
- [Branches and previews](branches-and-previews.md)
- [Workflows](workflows.md)
- [How WORTH works](../../../../../../docs/how-it-works.md)
- [Glossary](../../../../../../docs/glossary.md)
