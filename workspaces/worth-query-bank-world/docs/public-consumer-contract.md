# Public Consumer Contract

## Dependency boundary

Bank product code consumes Query only through `worth-query-decl` and
`worth-query-host`. It does not import Query authority packages, the Query
monolith, replay, Relational, Signal, or the runtime bridge.

Runtime replay remains confined to a later certification consumer.

## Read target

```rust,ignore
let bank = installed.bind(BankSchema::declaration())?;

let accounts = bank
    .query(Principal::reference())
    .traverse(PersonalOwner::reference())
    .project(AccountIdentity::reference())
    .project(AccountDisplayName::reference())
    .where_equal(Status::reference(), AccountStatus::Open)
    .build()?;
```

The ordinary Bank facade derives an account balance from authorized
`JournalPosting`, `PostingAccount`, and `PostingAmount` facts. Balance is not a
stored Account field and cannot be mutated independently of its journal.
Ordinary mutation preparation consumes Query's version-bound typed aggregate
summary: a checked signed value plus exact posting count under exclusive
`PostingAccount` cardinality. Bank requires that count to equal the governed
`AccountingRevision`; the warm summary is disposable and a cold graph rebuild
must recover the same result from posting truth.

## Mutation target

```rust,ignore
let transfer = bank
    .operation(SendMoneyOperation::reference())
    .input(SendMoney {
        from,
        recipient,
        amount: Money::<USD>::from_minor(cents)?,
    })
    .create(JournalEntry::reference())
    .create(Posting::reference())
    .set(PostingAmount::reference(), debit)
    .build()?;
```

## Domain effect target

```rust,ignore
let effects = bank
    .effects(SendMoneyOperation::reference())
    .emit(
        AccountActivityEffect::reference(),
        ActivityEvent {
            account: from,
            journal,
            journal_sequence,
        },
    )
    .build()?;
```

The payload type of every declared effect implements
`ApplicationEffectPayload`. Its `retained_bytes` result counts the inline value
and recursively owned allocation capacity. Query totals those bytes while the
effect program is authored and rejects the program before provider mutation if
it exceeds the installed operation envelope; live buffering cannot turn a
batch-count bound into unbounded payload memory.

## Approval and authorization authoring targets

```rust,ignore
let approval = bank
    .operation(ApprovePaymentOperation::reference())
    .input(ApprovePayment { payment, approver })
    .set(PaymentStatusField::reference(), PaymentStatus::Committed)
    .build()?;

let authorization = bank
    .operation(GrantAccountAuthorizationOperation::reference())
    .input(GrantAccountAuthorization {
        account,
        principal,
        role: CustomerRole::Viewer,
    })
    .create(AccountAuthorization::reference())
    .set(AuthorizationRole::reference(), CustomerRole::Viewer)
    .build()?;
```

These four blocks are Phase 1 compile targets and are exercised by
`public_consumer_transcript.rs`. They author installed-bound declarations; they
do not claim authentication, policy admission, execution, or commit authority.

## Ordinary execution target

```rust,ignore
let summary = runtime
    .query(queries::account_summary(account))
    .as_principal(&principal)
    .controls(BankReadControls::current(read_request, 32, 20_000)?)
    .execute();

let transfer = runtime
    .mutate(mutations::send_money(input))
    .as_principal(&principal)
    .controls(BankMutationControls::new(mutation_request, idempotency_key))
    .retain_operation()?;
let advancement = transfer.advance(
    &runtime.request(&fresh_principal, &fresh_scope), maximum_history_work,
);
```

Ordinary reads and the remaining native mutation callers supply a typed operation,
a fresh authenticated principal and caller-owned controls. The runtime owns
admission, projection, provider work, and outcome construction.
The caller cannot insert a read set, proposal, provider session, or receipt.

### Own account, access and payment completion

Personal and business account creation, access grant/revocation, payment initiation
and rejection use `retain_operation()` after their existing `mutate(...).as_principal(...).controls(...)` builders.
They return an owned native SourceCompletion operation. Construction transfers the
original input, typed conditions and idempotency key, and retains no principal or
request scope. Each `advance(&fresh_request)` opens one native phase; preserve the
returned owner on progress or refusal. A completed or terminal owner must be
closed with fresh authority, and a refused close returns its custody.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let operation = runtime
    .mutate(mutations::create_personal_account(input))
    .as_principal(&teller)
    .controls(controls)
    .retain_operation()?;
let outcome = operation.advance(&fresh_request);
```

Fresh completion carries the actual account, authorization or payment handler
result. Recovered completion carries the authoritative receipt and native
recovery evidence; it does not manufacture the original domain result. Read the
published account or payment with independently fresh query authorization.
`BankPendingPaymentContinuation::from_summary` constructs only a descriptive
payment identity from an authorized pending-payment read. Its rejection builder
resolves the supplied actor into the original rejection input; every subsequent
attempt must still pass native authorization and actor checks. Approval remains
in the approved-payment workflow, and a continuation grants no approval authority.

Receipt output correspondences describe created, preserved or retired entities.
They do not erase required-output source inventory: installed required-output
actions must use their declared fixed/discovered lane. These six completion
bindings share the installed Bank application and existing role policies.

### Own estate sources, funding, reversal and deployment revocation

Estate notification, retransmission, freeze, case opening, executor recognition,
release and disbursement use their typed `mutations` constructors and
`retain_operation()`. Each advance admits the binding's installed capability
against the current selected program; the owner retains the original command,
key and typed conditions. A receipt correspondence or `NoOutputGraph` does not
substitute for the actual installed output-source inventory.

Capability request selectors use the installed operation's scope and decision-read
contracts. Declare related/context selector fields through `operation_read_field`
(or an existing whole-entity decision read); governed elevation uses its installed
identity field. Query checks this footprint before resolving the selector. During
program adoption, the native candidate maintains these exact dependencies and
capability grant joins from its authentic predecessor. This preparation grants no
permission: an ordinary action removed by the successor still refuses, while an
exact performed-effect handle requires fresh authorization for recovery.

Funding and journal reversal retain their existing RetainedCommit delivery:
every advance takes fresh request authority and an explicit history-work bound.
The returned observation is the genuine original World basis. Deployment grant
revocation uses `retain_program_deployment_revocation` with its original u64 key
and typed conditions; a recorded issuer relationship, not an employee role,
authorizes the fresh attempt.

After completion, `close_with_evidence(&fresh_request)` releases the same native
cleanup collection before transferring `(closed, completion)`. On refusal, keep
the returned owner. `BankCommitReceipt::from_closed_source` publishes the original
receipt from that tuple; cloning borrowed descriptive evidence cannot transfer
the performed-change witness. Fresh handler results remain distinct from recovered
receipt and publication evidence.

Only after source completion and lawful close may notification/disbursement
receipts enter the existing published-effect recovery APIs. Those native handles
own outbox/rail reconciliation or compensation; they are not permission to rerun
the source handler. Source cleanup uses its native issuer custody and does not
reacquire revoked business permission. Business retries and redispatch still
require their applicable fresh native authorization. Elevation, delegation and
approved-payment workflow transitions keep their separate issued native owners.

A freeze command enters through the same ordinary mutation frontdoor. Construction
retains the command; it does not admit the business capability or execute an effect.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let owner = bank.mutate(mutations::freeze_estate_account(input))
    .as_principal(&specialist)
    .controls(BankMutationControls::new(scope.clone(), key))
    .retain_operation()?;
let outcome = owner.advance(&bank.request(&specialist, &scope));
```

For a publication interruption, retain the native `Progress` owner. A fresh
unauthorized recovery returns that owner in `Refused`; recovery under an authorized
request returns progress and then `Completed` with `Recovered` evidence. It supplies
no replacement handler result. The [owning estate courts](../crates/bank-server/tests/ordinary_mutations/estate_operations/owning_sources.rs)
exercise this flow, independent account truth, refusal during close, and witness transfer.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
match completed.close_with_evidence(&bank.request(&specialist, &fresh_scope)) {
    Ok(closed_completion) => {
        let receipt = BankCommitReceipt::from_closed_source(closed_completion);
        // Published-effect recovery consumes this receipt through its own native owner.
    }
    Err((completed, denial)) => {
        // Retain completed, including its original evidence, for a fresh lawful close.
    }
}
```

### Own a retained deposit

The deposit builder also provides `retain_operation()`: call it after
`mutate(mutations::deposit(input)).as_principal(&principal).controls(controls)`.
It transfers the original input, typed preconditions and owned idempotency key
into the installed `BankApplication` deposit action. Construction retains no
principal or request scope. The five money actions declare distinct retained
delivery contracts. Deposit, withdrawal and send-money builders expose owning
construction; opening funding and reversal consumer cutovers remain separate.

Advance the returned operation with `operation.advance(&fresh_request,
maximum_history_work)`, using `runtime.request(&fresh_principal, &fresh_scope)`
and an explicit `NonZeroUsize` history bound for that attempt. Keep the owner
returned by `Progress` or `Refused`. Native unpublished recovery uses fresh
authorization and adopts the original journal/posting effects without executing
the deposit handler again. A fresh stranger cannot recover a teller's deposit.

On completion, `Fresh` evidence includes the actual `MoneyMovementResult`;
`Recovered` evidence includes native receipt/publication evidence and the original
retained World observation, without recreating the handler result. Use that
observation with the saved account activity request's `retained` read under fresh
read authority. Later deposits change current activity without retargeting the
original retained observation. Keep completed custody if `close(&fresh_request)`
refuses and retry with lawful fresh context.

The [owning deposit court](../crates/bank-server/tests/ordinary_mutations/owning_deposit.rs)
executes the real append-partial, unauthorized recovery, authorized recovery,
historical read and cancelled-close/fresh-close journey. It also verifies all five
installed money deliveries and preserves the deployed program capability meaning.
HTTP stores these actual owning money operations in finite transport custody.
Use the [owning HTTP money contract](http-owning-money.md) for create, resume,
historical read, close and independent listener-loss continuation. Remaining
Bank consumers still require their owning cutovers. Do not hide advancement in a
loop or reconstruct original meaning to discard returned custody.

## Workflow, history, and live target

```rust,ignore
let initiation = runtime
    .mutate(mutations::initiate_business_payment(input))
    .as_principal(&initiator)
    .controls(initiation_controls)
    .execute();

if let Some(pending) = initiation.continuation() {
    let approval = runtime
        .mutate(pending.approve())
        .as_principal(&fresh_approver)
        .controls(approval_controls)
        .execute();
    inspect(approval.explanation());
}

let page_request = request_scope();
let first_page = runtime
    .account_activity(account)
    .as_principal(&principal)
    .page(WorthQueryApplicationQueryControls::current_continuation_page(
        NonZeroUsize::new(25).unwrap(),
        NonZeroUsize::new(4_096).unwrap(),
        &page_request,
    ))?;
let (rows, next, receipt) = first_page.into_parts();

if let Some(next) = next {
    let resume_request = request_scope();
    let second_page = runtime
        .account_activity(account)
        .as_principal(&principal)
        .resume(next, WorthQueryApplicationQueryResumeControls::new(
            NonZeroUsize::new(25).unwrap(),
            NonZeroUsize::new(4_096).unwrap(),
            &resume_request,
        ))?;
}

let mut activity = runtime
    .account_activity(account)
    .as_principal(&principal)
    .subscribe(WorthQueryApplicationLiveControls::bounded(
        live_request,
        16,
        8,
        2_048,
    )?)?;
match activity.poll() {
    BankAccountActivityLiveOutcome::Delivered(update) => publish(update.result()),
    BankAccountActivityLiveOutcome::Overflow(missed) => resynchronize(missed),
    BankAccountActivityLiveOutcome::AuthorizationDenied(_) => close_client(),
    other => handle_terminal(other),
}
```

A payment continuation is descriptive. It can cross a process boundary, but it
does not retain the initiator's authority. Approval or rejection always starts
again with a fresh authenticated principal and fresh controls.

An activity continuation is an opaque, move-only description of the next
ordered page. It contains no permission or retained runtime resource. Each
resume supplies a fresh authenticated principal, account scope, request
deadline, cancellation token, page width, and work bound. Query reacquires the
original provider version, rechecks current account permission, and seeks from
the exact last-row boundary through the installed ordering index. A commit
between pages therefore does not mix new rows into the sequence, while a
permission revocation denies the next page.

Live delivery is bounded. A slow consumer receives typed overflow and must
resynchronize through the same installed ordinary query. Permission is checked
again before each payload, so queued data does not survive revocation as
authority. One query-shaped result carries the exact posting targeted by the
committed domain cause; the host neither filters nor reconstructs history. A
delivered activity update contains the exact newly caused activity item, not a
bounded snapshot that may omit it; the bank facade retains the typed cause
until the fresh authorized one-item projection succeeds.

## Branch program inspection, adoption, and recovery

The Bank server exposes the Query-owned branch-program path through domain-named
methods rather than accepting revision strings or World handles:

```rust,ignore
let before = runtime.inspect_branch_program(&principal, &scope, branch)?;
let prepared = runtime.prepare_branch_program_adoption::<BankApplicationP1>(
    &principal,
    &scope,
    branch,
    128,
)?;
let outcome = prepared.publish();
let after = runtime.inspect_branch_program(&principal, &fresh_scope, branch)?;
```

`BankApplicationP1` must already be rostered by the installed Bank host.
Preparation compares the branch's actual source program with that target,
derives state/migration and continuation/resource custody, and validates the
target rules before publication. Only a performed World outcome changes what
`inspect_branch_program` reports. Ordinary Bank mutations select their handler
through that exact branch occurrence; an operation removed by P1 returns
`ProgramNotActiveOnOccurrence` without producing an effect.

Already-performed external work remains bound to its original P0 occurrence.
The production recovery handle continues through the ordinary Bank recovery
root after P1 removes the operation, with the same outbox correlation and no
second business effect. Query's unpublished adoption recovery is a separate
move-only carrier; it preserves source/target support custody and cannot be
reconstructed from Bank diagnostics. The executable production-root claims are
pinned by `workflow_continuations.rs` and
`ordinary_mutations/estate_operations/program_adoption_recovery.rs`.

## Controls and outcomes

Read controls own consistency, deadline, cancellation, result limits, and
bounded work. Mutation controls own deadline, cancellation, and idempotency.
Live controls own the read bound plus caller-narrowed delivery buffering.

Outcomes retain typed denial, invariant, stale, abort, cancellation, deadline,
partial-effect, indeterminate, committed, and recovered meaning. Use
`explanation()` for presentation; do not parse diagnostic text.

A mutation retried under its idempotency key after the server restores or
reopens is still recognized. A changed request under the key is refused as
stale with `correct_request`. The unchanged retry commits nothing; when the
server no longer holds the original receipt it answers `stale` with `refresh`,
because the original commit already took effect.

## Process transport

`bank-http-adapter` now provides the authoritative Axum HTTP/SSE process and
`bank-user-node` provides one independently authenticated client process per
participant. Both bind dynamic ports, report typed `bound` and `ready`
postures, accept one named JSON installation document over stdin, and shut down
through the typed process command. Production callers use the versioned
`BankHttp*` and `BankUserNode*` request/outcome families rather than parsing
status text.

The user node stores only its Authentik credential and forwards fresh requests.
The Bank server retains continuation, recovery, and elevation authority behind
bounded opaque tokens. Elevation approval, revocation, and
mandatory review accept only that token plus a fresh credential, deadline, and
idempotency key; callers cannot transmit estate, branch, grant, phase, or Query
authority at those transitions. Live account activity is translated to SSE
without moving the underlying Query lease into the user node.

The principal supported endpoints are `/v1/queries/account-summary`, the
account-activity page/resume and live paths, `/v1/mutations`, the estate
notification and disbursement paths, recovery inspection, and the four
`/v1/estate/elevation/*` transitions. The same application request shapes are
available at a user node without a credential field; that process supplies its
own authenticated session. Linear undo/redo routes remain provisional
Milestone 9.18 experiments rather than a Bank Phase 5 product contract.

Every Bank server request body declares `"protocol": "v4"`. The server admits
that one version and refuses earlier versions before effects. Money commands
retain actual native custody and distinguish pending, refusal, completion,
terminal custody and a queued command with unknown outcome. A timeout cannot
assert that a mutation did not land. Route paths retain their `/v1/` prefix.

Estate `notify-death` and disbursement first return owning SourceCompletion
custody. Fresh resume, physical release and close commands retain that owner;
only its actual completed witness establishes the original dispatch commit.
Approved-payment responses follow their separate workflow-owned dispatch path. The rail can report `Completed` synchronously, or send the
same effect's signed completion through Bank's private callback route later.
Both observations meet at the original Query owner and one World terminal.
The approved-payment workflow's `await_inbound` node names that payment
operation and its installed settlement effect. A callback never advances the
instance; a fresh authenticated workflow advance observes the owner result.
Cancellation or definition retirement cannot abandon an operation still in
owner custody. The
[compiled payment definition](../crates/bank-server/src/application_definition/workflows.rs)
and [process court](../crates/bank-courtroom/tests/transport_process_courtroom/payment.rs)
exercise those paths.
Clients do not submit callback bytes or interpret a rail transport ACK as a
completed estate operation. Bank acknowledges callback custody only after
Query authenticates and correlates the exact original dispatch. Its signed
postures are `AcceptedPending`, `AlreadyAccepted`, `Performed`, and
`AlreadyCompleted`; unavailable or revoked recovery receives no signed ACK.
The rail retains immutable signed bytes through bounded retry and reports
exhausted obligations for reconciliation. HTTPS installation requires pinned
Bank TLS trust plus a distinct pinned Bank ACK key. Both rail delivery and
Query accepted custody are process-local in this phase; orderly rail close
reports unresolved obligations, while forced process death has no persistence
promise.

Bank Phase 5 is closed. The Docker-backed Authentik courtroom executed the
complete separate-process failure matrix through the production Bank server and
independent user-node binaries; its evidence is recorded in
`front-door-closure-ledger.md`. Runtime Phases 9 and 10 are also closed: the
host conditional-operation surface is complete, the audience facades are
contracted, and public cutover residue is enforced. Bank Phase 6 is the next
milestone frontier and must compose the complete consumer journeys through this
same public contract.

## Prohibitions

Application paths contain no semantic aspect, field, relation, operation,
policy, or currency strings. A dynamic extension, if admitted later, uses an
explicit dynamic key and schema-readmission result. Caller-cast values and raw
`AspectValue` construction are not the ordinary bank API.

Do not retain an authenticated principal, workflow continuation, or activity
continuation as a substitute for a new request. Do not copy pagination
identity into an offset or rebuild it from result values. Do not retry overflow
from the live cursor; resynchronize with an ordinary query.
Do not interpret a successful proposal as a commit—the public mutation outcome
is the authoritative terminal surface.
- Do not choose a program in an HTTP route, deserialize a program revision as
  authority, or retry an operation removed by the selected branch program.

## Related docs

- [Bank Process Transport](process-transport.md)
- [Banking Product Contract](banking-product-contract.md)
- [Async Identity Courtroom](async-identity-courtroom.md)
- [Front-Door Closure Ledger](front-door-closure-ledger.md)


## Retain native HTTP cleanup evidence

The listener-independent mutation continuation owns actual SourceOnly money and
estate operations, plus native cleanup returned by the four elevation transitions.
Every command authenticates freshly. `release_cleanup` releases the native physical
collection on its original execution thread; it never asserts a business commit.
A release refusal preserves the original owner, and pending cleanup blocks shutdown.
The local host handoff `take_released_cleanup` transfers original uncommitted evidence
with an exact installed-runtime keeper after lawful physical release. Reply loss
before host receipt returns the evidence through a reserved transport slot.
See [owning HTTP commands](http-owning-money.md#estate-and-native-cleanup-custody)
for current wire and host usage. Abandonment has no disposal or crash-durability promise.
