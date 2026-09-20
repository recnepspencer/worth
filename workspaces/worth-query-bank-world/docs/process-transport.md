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
