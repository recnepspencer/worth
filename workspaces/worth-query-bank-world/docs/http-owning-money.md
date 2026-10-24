# Owning HTTP mutations

## What this feature is

Deposit, withdrawal, send-money, estate freeze, capability revocation and recorded
correction commands retain their Query operation between HTTP requests. Query
owns admission, source execution, recovery, delivery and close. The HTTP custody
ID addresses that operation throughout its lifecycle.

## Why you use it

A lost response does not lose the original operation. Choose a custody ID before
sending create, then address that custody with fresh authenticated commands.
Authenticate each command and keep the original idempotency key with its input.

## Stable entry points

`BankHttpServerBinding::install` and the server bind methods return
`BankHttpServerInstallation`. Its `into_parts()` returns a `BankHttpPreparedListener` and
an independent `BankHttpMutationContinuation`. Retain the latter in the host's
lifecycle owner, then call the prepared listener's `activate()` to accept requests.
The continuation's `execute` accepts the same
strict v4 request as `POST /v1/mutations`, including fresh credentials.
User-node requests carry the same command fields without protocol or credentials;
the node supplies its current session credential for each command.

## Core mental model

Create supplies the original typed mutation input and native idempotency key.
Optional send-money accounting revision and account-status conditions are encoded
through the Bank typed precondition builder. That captured input remains with the
operation. Each advance and retained read checks current business authorization.
Cleanup uses runtime-issued release authority and a freshly admitted request.

Commands for the same ID route to one persistent worker on a dedicated execution
thread. Each accepted command increments
that record's sequence exactly once. A stale or duplicate sequence performs no
second advancement and replays no protected query rows. Native refusal retains
the returned owner; a sequence refused before record admission is not consumed.
An authorization refusal before the first effect leaves native `Initial` custody.
Use fresh close controls on the same ID to release it lawfully and return its
capacity; partial and uncertain publication remain owned for recovery.

## How it executes

One create or resume command invokes at most one native advance. Supply explicit
nonzero `maximum_history_work` on each such command. Pending responses identify
native posture; another fresh command advances it. Fresh completion and recovered
receipt evidence are distinguished by `recovered`. Money actions retain their
original publication for activity reads. Read supplies an explicit `account` and uses the original
retained World observation with fresh query authority and `maximum_results`.
The account chooses which activity to query; the retained operation chooses the
publication at which that activity is read.

The worker stores every returned native owner before sending its response.
After queue admission, timeout, cancellation or reply loss may leave the result
unknown. Keep the custody ID and use fresh commands; do not issue a replacement
create as a safe rerun. Pre-admission rejection reports no execution by that
command, rather than making a claim about an earlier operation with that key.

## Small example

```json
{
  "protocol": "v4", "command": "create", "request_id": "deposit17",
  "credential": { "id_token": "...", "access_token": "...", "nonce": "..." },
  "custody_id": "client-chosen-deposit17", "sequence": 1,
  "controls": { "deadline_milliseconds": 5000, "maximum_history_work": 128 },
  "idempotency_key": "original-deposit17",
  "operation": {
    "operation": "deposit", "institution": "fixture:1",
    "account": "fixture:100", "amount_minor_units": 17
  }
}
```

## Real example

A completed money command returns `source`, the exact occurrence's descriptive
coordinates. To correct a closed send-money occurrence, create a new custody
with a fresh idempotency key and `operation: { "operation": "correct_send_money",
"source": source }`. Current teller authority and account disclosure are checked
before the installed journal reversal runs. The original and compensating
journals remain available in account activity, including intervening movements.

Use the same source and key to recover an uncertain correction result. Completion
returns the original correction occurrence. To inspect its retained activity
after later transactions, send:

```json
{
  "command": "read", "request_id": "correction-activity",
  "custody_id": "client-chosen-correction", "sequence": 3,
  "account": "fixture:101",
  "controls": { "deadline_milliseconds": 5000, "maximum_results": 16 }
}
```

This user-node request uses the current authenticated session. Direct Bank HTTP
requests additionally include `protocol` and `credential`. The response's
publication basis identifies the retained correction version.

After losing an append-partial response, resume the same ID with sequence 2,
fresh credentials and a funded history bound. Native recovery adopts the original
publication without posting money again. Continue with one explicit command per
returned pending posture. Once completed, read under fresh query authority to
observe the original commit even after later activity changes current truth.
Close with fresh controls; if refused, retain custody and retry lawfully.

Shutdown first quiesces new creates. While native custody remains, shutdown
returns `BankHttpServerCloseFailure`; `into_server()` recovers its still-live
listener and resume routes. The independent continuation also works after the
listener or failure wrapper is dropped. Close all records lawfully before a
successful shutdown joins the mutation workers. After listener loss,
`continuation.finish().await` joins only when all native records are closed;
refusal returns the continuation. Rail completion custody retains
its separate handoff contract.

## Choosing and reapplying historical SendMoney

Keep completed mutation custody for each occurrence that later historical
selection or retry will need. Its retained native observation keeps that exact
World occurrence available. For a source S, reversal U, reapplication R and later
reversal V, retain their completed custody IDs until the chooser and command
retries finish, then close each with its next sequence. Each owner retains its
own occurrence. The history token retains traversal progress. Keep the completed
owners alive for the occurrences the journey needs.

After reversing a SendMoney occurrence, inspect its alternatives with
`POST /v1/corrections/history`. Supply the original completed `source`, a request
ID and explicit history, disclosure and page bounds:

```javascript
const historyRequest = {
  request_id: "choose-original-transfer",
  source: transferResponse.source,
  controls: {
    deadline_milliseconds: 5000,
    maximum_history_work: 128,
    maximum_disclosure_results: 16,
    maximum_entries: 1
  }
};
```

The user-node supplies its current session credential. Direct Bank requests also
carry `protocol: "v4"` and `credential`. Each delivered page contains
`alternatives`, a `continuation` token and `complete`. Save the desired row's
`source`. While `complete` is false, send the same source and token to
`POST /v1/corrections/history/resume` with a new request ID and fresh controls.
Page width and disclosure-query results have separate bounds. Repeating the
initial or latest continuation request ID returns its recorded response after
current disclosure succeeds.

Once traversal is complete, create ordinary mutation custody for the chosen row:

```javascript
const operation = {
  operation: "reapply_send_money",
  history_token: finalPage.continuation,
  source: chosenRow.source
};
```

Use a fresh idempotency key for a new reapplication. Query resolves the row against
canonical history and executes the captured original SendMoney input under
current source authority and account truth. Reapplication appends a new journal;
the original movement and its reversals remain visible. Advance its ordinary
custody, read retained activity if needed, and close it with the next sequence
when its historical use finishes.

The chooser token belongs to the authenticated participant and expires under the
server's configured handle lifetime. A branch advance makes its native resolution
stale. Refresh history to choose again. For an exact command retry, select the
same historical row with refreshed history and retain its original key; native
retry returns the recorded occurrence even after another lineage transition.
After losing a mutation response, resume its saved custody ID. Chooser expiry
leaves that mutation owner retained for advancement and normal close.
Once the completed history owners close, later publications can retire their
optional source history. A fresh chooser then returns `stale` with `refresh`
when that source is unavailable.

## Correcting an estate freeze

Create `freeze_estate_account` with an estate ID and canonical account ID:

```json
{ "operation": "freeze_estate_account", "estate": 3, "account": "fixture:100" }
```

The completed response supplies the exact freeze occurrence in `source`. Close
that custody with the next sequence. To restore its recorded account status,
create a new operation with a fresh key:

```javascript
const operation = {
  operation: "correct_estate_freeze",
  source: freezeResponse.source
};
```

Restoration requires the account still to be Frozen and current authority for
that estate/account.
Both commands complete with a receipt; use an ordinary account-summary query to
observe the current status, then close the completed custody. A revoked grant
causes fresh correction admission to return `permission_denied`.

An authorized issuer revokes a grant through `revoke_estate_capability` with its
estate and grant IDs. Resume pending custody with fresh controls and close after
completion. Revocation uses the same command sequencing and recovery lifecycle.

## Estate effects and native cleanup custody

Estate notification and disbursement construct declared SourceCompletion owners.
Their bounded, client-chosen request ID is the initial custody address (sequence
1), independent of the native key and permissions. An `owning` response contains
the actual operation posture. Use `resume_source` with fresh effect controls;
this no-output declaration needs no retained history argument. A successful
`close` of a completed estate owner uses `close_with_evidence` and transfers its
original witness to separate rail recovery. `rail_recovery_ready` reports the
actual recovery token or the native completed/operator-required claim posture.
It does not manufacture the handler's result after recovery.

`release_cleanup` invokes the native owner's physical cleanup under a fresh
bounded issuer. `collection_released` reports release of that cleanup collection.
Continue managing the original source owner according to its publication posture.
A failed release leaves custody in its worker for a fresh attempt.

Four elevation transitions can return `cleanup_required`. Their actual native
owner and original uncommitted evidence occupy the same finite transport custody.
Their lifecycle continues through cleanup and close. Execution and physical retry remain
on the original dedicated thread. After `cleanup_released`, the host calls
`take_released_cleanup` with fresh authenticated close controls to receive a
move-only `BankHttpReleasedCleanup`. The reserved transport delivery guard returns
that exact evidence if the handoff response is not consumed; shutdown sees its
outstanding custody. `into_parts()` transfers a `BankHttpCleanupRuntime` keeper
and the actual `BankReleasedManagedCleanup`, whose physical receipt remains
separate from its original business outcome. Further business transitions require
their own current authority.

<!-- illustrative -->
```rust,ignore
let handoff = continuation.take_released_cleanup(fresh_close_command).await?;
let (keeper, evidence) = handoff.into_parts();
let original = evidence.business();
let physical = evidence.receipt();
// Retain keeper while using the actual runtime's fresh inspection/recovery entrances.
let runtime = keeper.runtime();
```

Listener-independent `inspect_recovery` and `retry_recovery` delegate to the
existing estate recovery registry under fresh credentials. The continuation keeps this registry and installation
alive across listener loss; a queued rail retry can also return `outcome_unknown`.

## How it relates to other features

Payment continuation tracks a business payment; estate recovery owns external
effect custody; account-page cursors own query continuation. Each lifecycle has
its own operations and completion evidence. Money retained reads use the original
native history observation to return the exact publication's account activity.

## Inspection and debugging

Wire outcomes distinguish `rejected`, `outcome_unknown`, `pending`, `refused`,
`completed`, `terminal`, `delivered`, physical collection release, rail handoff
and native `closed`. A close refusal does not discharge custody.
Duplicate live custody IDs and exhausted capacity reject before another effect.
A full command queue reports `saturated`; a closed worker reports `unavailable`.
`continuation.runtime()` lends the exact retained installation for independent
native catalog and cleanup access. It cannot reconstruct lost transport records.
Strict DTOs reject earlier protocol versions and fields belonging to a different
command. The native operation remains the authority for terminal and retry truth.

## Host lifetime

Custody lives in the running Bank process. Keep the host continuation and original
operation records until each operation and cleanup obligation closes. When a
queued request times out, resume its custody with fresh credentials and controls
to discover the actual outcome. Pending operations remain owned through recovery.

Complete cleanup and close before destroying the runtime. Dropping the final
continuation closes ingress and lets workers end; explicit native cleanup owns
resource release. World catalog discovery and managed cleanup collections expose
their respective live obligations while the runtime remains available.
Process-crash durability requires a separate host persistence contract.

## Related docs

See [the Bank consumer contract](public-consumer-contract.md) and the actual
[owning HTTP courts](../crates/bank-http-adapter/src/http/server/tests/mutation.rs).
