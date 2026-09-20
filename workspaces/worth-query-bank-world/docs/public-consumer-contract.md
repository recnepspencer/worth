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
    .execute();
```

Every ordinary call follows the same shape: choose a typed operation, supply a
fresh authenticated principal, supply caller-owned controls, then execute. The
runtime owns admission, projection, provider work, and outcome construction.
The caller cannot insert a read set, proposal, provider session, or receipt.

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
