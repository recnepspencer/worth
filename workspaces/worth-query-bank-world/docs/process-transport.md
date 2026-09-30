# Bank Process Transport

## What This Feature Is

The Bank process transport runs one authoritative Bank server behind HTTP/SSE
and gives each signed-in participant a separate user-node process. Application
clients talk to their own node; the node supplies its stored Authentik
credential and forwards typed requests without receiving Query runtime
authority.

## Why You Use It

- Keep Bank state and policy in one authoritative server process.
- Isolate each participant's browser login and credential in its own node.
- Carry query results, live updates, continuations, recovery, and elevation
  progression over ordinary JSON and SSE boundaries.

## Stable Entry Points

- `bank_http_adapter::run_bank_http_server_process()` runs the authoritative
  process protocol.
- `bank_user_node::run_bank_user_node_process()` runs one participant node.
- `BankHttp*` types are the server wire contract.
- `BankUserNode*` types are the credential-free node request and outcome
  contract.
- `BankHttpServerBinding` and `BankUserNodeBinding` provide embedded process
  composition when the caller owns lifecycle orchestration.

The `cold-certification` feature changes only TLS trust for the disposable
Docker courtroom. It is not a production deployment mode.

## Core Mental Model

The Bank server owns domain truth, authorization, provider work, and opaque
lifecycle authority. A user node owns one Authentik credential and a bounded
HTTP client. Wire values describe outcomes; they cannot resume Query execution
or mint authority by themselves.

An opaque continuation, recovery, or elevation string is a lookup key into a
bounded server registry. Using it always requires a fresh authenticated
request. A query publication envelope describes the exact query identity,
parameter binding, basis, capability purpose, and disclosure or omission
posture without exposing an executable handle.

The current linear undo/redo routes are provisional experiments retained for
regression work toward Milestone 9.18. They are not a supported transport
contract and do not close any Bank Phase 5 obligation.

## How It Executes

1. Start the Bank server. It binds a dynamic or configured address and reports
   `bound` on stdout.
2. Send one JSON installation document on stdin. The process installs the
   Authentik adapter and Bank world, then reports `ready` with its PID/address.
3. Start one node per participant. Each node follows the same
   `bound -> install -> ready` progression and points at the Bank origin.
4. POST `/session/authorize` at the node and complete the returned Authentik
   browser flow. The callback installs the credential only in that node.
5. Send credential-free requests to the node's `/v1/*` endpoints.
6. POST `/session/revoke` to revoke the access token, clear the node session,
   and cancel active live responses.
7. Send `{ "command": "shutdown" }` on stdin for deterministic shutdown.

Requests and active streams have separate concurrency ceilings. Deadlines
cover node-to-server connection, headers, JSON bodies, and the full SSE
lifetime. Capacity is reserved before a recovery-producing domain effect.

## Rail Completion Callback

`NotifyDeath` and approved-payment settlement each commit one external
dispatch before the separate rail can complete its consequence. The rail sends
a signed v1 completion to the Bank
server's private `POST /v1/inbound/rail-completions` route. The route installs
one rail Ed25519 verification key, audience, source (`rail-primary`), and key
epoch. Callback bytes cannot select another verifier or operation. Bank checks
the signature and exact original dispatch through Query before signing a
custody ACK with a separate Bank key.

The signed ACK binds its posture to the rail message ID and SHA-256 of the
complete signed callback body. `AcceptedPending` means Query retained accepted
evidence, including exact World recovery if effects are unpublished.
`AlreadyAccepted` repeats that custody result. `Performed` and
`AlreadyCompleted` report World terminal truth. Bank currently sends no signed
permanent denial. A retry-before-acceptance, revocation, capacity refusal,
publication failure, or HTTP error returns no signed custody ACK. The rail
resends the same signed bytes with bounded attempts and backoff; exhaustion
keeps an observable unresolved sender obligation for reconciliation.

The installed Bank server runs one serial, bounded maintenance task for Query
custody. It wakes when accepted work is retained, on explicit host continuation
after an external recovery change, or at the next terminal expiry; it does not
poll an idle inbox. It continues an already accepted completion after the
sender's signed envelope expires and awaits its current batch on orderly
shutdown. Its
`observe_rail_completion` method is a route-bound host diagnostic on the
server handle; there is no raw-correlation HTTP inspection route. An immediate
rail `Completed` response also enters Query's World completion path with
transport provenance, and the later signed callback receives
`AlreadyCompleted` for the same effect.

The v1 callback body is the exact bytes below, followed by a 64-byte Ed25519
signature over the preceding bytes. Integers are unsigned big-endian; text is
UTF-8 preceded by its `u16` byte length. The token uses a `u16` byte length
and the payload uses `u32`. No field is optional or extensible in v1.

| Order | Field |
| --- | --- |
| 1 | `BANK-COMPLETION1` (16 bytes), audience, source, key epoch (`u64`) |
| 2 | Message ID (32 bytes), issued and expires Unix seconds (`u64` each) |
| 3 | Effect protocol identity, version (`u16`), correlation family, token, exact effect payload |

The rail derives the message ID as SHA-256 of `bank-rail-completion-id-v1`,
then the length-prefixed audience and source, key epoch, protocol identity and
version, correlation family and token, and payload in that order. This identity
stays stable across retries. Bank's ACK signs `BANK-CUSTODY-ACK` (16 bytes),
the one-byte posture, message ID, and SHA-256 of the complete signed callback
body. The rail verifies both that signature and the exact message/digest; an
HTTP 200 or altered ACK cannot release its sender obligation.

The first compatibility window accepts v1 only. New versions require explicit
coexistence and retirement rules at installation; v1 bytes are never
reinterpreted or downgraded by a caller. Bank verifies the installed audience,
source, epoch, signature, protocol and original dispatch. A message remains
acceptable through its signed expiry second, subject to the installed clock
skew and replay-window ceiling; after expiry it cannot create new custody.
Already accepted evidence continues through bounded maintenance without
reauthenticating expired bytes. Pending or unpublished custody is never evicted
at a request deadline or replay cutoff.

The installed Bank declaration caps an envelope at 4,096 bytes and verifier
work at 20,480 abstract units. Its verifier charges four bounded byte passes
and one fixed signature operation (`4 × envelope length + 64`) and refuses a
budget excess before authentication. An arbitrary product verifier must honor
its own installed work budget; a byte cap by itself does not constrain
arbitrary code.

For HTTPS deployment, install one PEM Bank TLS trust certificate and the
separate Bank ACK verification key in the rail sender configuration. Reqwest
uses only that installed root and does not follow redirects. Loopback HTTP is
allowed for the process courtroom with `None` as the final argument:

```rust,ignore
let delivery = RailCompletionDeliveryConfiguration::new(
    "https://bank.example/v1/inbound/rail-completions".to_owned(),
    "bank-prod".to_owned(),
    "rail-primary".to_owned(),
    1,
    rail_signing_seed,
    bank_ack_verifying_key,
    60,
    128,
    8,
    Duration::from_millis(100),
    Duration::from_secs(5),
    Some(bank_tls_root_pem),
)?;
```

The Bank side installs the matching rail verification key, the Bank ACK
signing seed, and the same audience, source and key epoch through
`BankRailCompletionServerInstallation::new`. The installation creates fixed
estate and payment verifiers; callback bytes cannot add a protocol or replace
either verifier. The
[real process setup](../crates/bank-courtroom/tests/transport_process_courtroom/rail_completion.rs)
and [payment setup](../crates/bank-courtroom/tests/transport_process_courtroom/payment/setup.rs)
exercise both installed routes. Rotate keys and epochs through an explicit
deployment overlap and retirement policy; the current single-key process
installation does not negotiate versions or silently accept an old key.

`RailProcessHandle::close()` stops new rail contacts and returns
`RailCompletionDeliveryPosture { reserved, pending, exhausted }` after the
child prints `CLOSING` and exits. Pending includes exhausted obligations.
The Bank HTTP server's orderly `shutdown()` drains its listener and current
maintenance batch, then returns a `BankHttpServerClose`. Its rail assessment
reports pending recovery work, outstanding dispatch provenance, retained
accepted custody, blocked work, the next replay-cleanup expiry, and whether an
installed route could not be assessed. These owner views can overlap and must
not be summed. An unavailable assessment must not be read as zero outstanding
work. A shutdown failure also retains the close result through `into_close()`.
The close result can transfer the original installed route into a
`BankRailCompletionContinuation`; a surviving host may call `continue_once()`
after owner recovery without reinstalling a verifier or opening another HTTP
listener.
Dropping the handle forces termination and gives no delivery survival promise;
the sender and Query custody in this phase are process-local.

## Small Example

After browser authorization, query the participant's own account through their
node:

```http
POST /v1/queries/account-summary
Content-Type: application/json

{
  "request_id": "summary-42",
  "controls": {
    "deadline_milliseconds": 5000,
    "maximum_results": 1,
    "maximum_work": 20000
  },
  "account": "fixture:100"
}
```

A successful `BankUserNodeAccountSummaryOutcome::Forwarded` contains the typed
Bank response, including its authority-free query publication description.
Unknown fields and unsupported protocol versions fail closed.

## Real Example

Open a bounded activity stream through the node:

```http
POST /v1/live/account-activity
Accept: text/event-stream
Content-Type: application/json

{
  "request_id": "activity-42",
  "controls": {
    "deadline_milliseconds": 30000,
    "maximum_results": 8,
    "maximum_work": 20000
  },
  "account": "fixture:100",
  "source_buffer_capacity": 16
}
```

The stream begins with `opened`. Each `update` carries one Bank activity result
and its publication description. `overflow` means the client must resynchronize
through the page query. `cancelled`, `deadline_exceeded`, `closed`, and
`unavailable` are terminal. Revoking or replacing the node session cancels the
active response, and dropping the client response releases both the node and
server live permits.

Recovery and elevation journeys follow the same rule: retain the returned
opaque token, send it to the purpose-specific next endpoint, and supply a new
request ID, deadline, and idempotency key. Never decode or rewrite the token.

Branch-program inspection and adoption currently belong to the authoritative
same-process `BankIdentityRuntime` root. They are not HTTP/user-node commands.
The server methods `inspect_branch_program` and
`prepare_branch_program_adoption::<Target>` require a fresh authenticated Bank
principal and request scope, and the target must already be rostered. A route
must not accept a revision digest, prepared adoption, or recovery carrier as
JSON. External-effect recovery remains valid after an adopted program removes
the ordinary operation because the server retains the exact original
occurrence and outbox authority; the node still receives only an opaque
purpose-specific recovery token.

## How It Relates To Other Features

- Use the ordinary Bank facade directly for same-process embedding.
- Use this transport when identity/session isolation or real network/process
  boundaries are part of the product.
- Query owns continuation, live, recovery, and publication semantics. The
  adapter translates them; it does not reproduce them.
- Linear undo/redo remains provisional pending the separately governed
  tree-based product. Its transport types are usable but are not a promise of
  final history semantics.

## Inspection And Debugging

- Read the typed denial `kind` and `next_action`; never parse diagnostic text.
- Treat HTTP status as transport posture and the JSON outcome as semantic
  posture.
- Inspect `BankHttpQueryPublication` to distinguish public disclosure,
  governed disclosure, and governed omission.
- A restarted node intentionally has no authenticated session. Reauthorize it;
  do not copy credential or session state from the old process.
- `RequestSaturated` and `Saturated` are retryable only after capacity is
  released. `DeadlineExceeded` means the original attempt has ended.

## Anti-Patterns

- Do not send branch, provider, snapshot, generation, or Query authority fields.
- Do not put banking policy, default branch selection, or token decoding in a
  route or node.
- Do not treat an opaque token as authentication or authorization.
- Do not retry an SSE overflow from an inferred cursor; start a bounded page
  query and then reopen live delivery.
- Do not enable cold-certification trust in a deployed process.

## Current Limits

- The process protocol accepts one installation document and one shutdown
  command; orchestration and supervision belong to the deployment owner.
- The Docker/Authentik process courtroom is the final cold certification. On a
  host without Docker, the court can compile but cannot establish runtime
  evidence.
- The transport does not provide distributed Bank-server replication or
  authoritative failover.
- The transport exposes no program-adoption endpoint. Adding one requires a
  separately versioned protocol that preserves Query's move-only preparation
  and unpublished-recovery custody; raw revision IDs are insufficient.

## Related Docs

- [Public Consumer Contract](public-consumer-contract.md)
- [Async Identity Courtroom](async-identity-courtroom.md)
- [Banking Product Contract](banking-product-contract.md)
- [Front-Door Closure Ledger](front-door-closure-ledger.md)
