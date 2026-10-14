# Owning HTTP money commands

## What this feature is

Deposit, withdrawal and send-money commands retain their actual Query operation
between HTTP requests. The transport stores finite custody; Query decides
admission, source execution, recovery, original-commit delivery and native close.

## Why you use it

A lost response does not lose the original operation. Choose a custody ID before
sending create, then address that custody with fresh authenticated commands.
The ID is descriptive transport routing, not a permission or idempotency key.

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

Create alone supplies the original typed money input and native idempotency key.
Optional send-money accounting revision and account-status conditions are encoded
through the Bank typed precondition builder once. Resume, read and close cannot
replace these fields. The transport retains no principal, scope or grant proof. Business authorization
is freshly native on advance and retained read. Native cleanup uses its own
runtime-issued release authority and fresh admitted scope; an actor business
grant is not a substitute for those cleanup rights.

A record occupies one globally bounded permit before its first advance. Commands
for the same ID route to one persistent worker. Each accepted command increments
that record's sequence exactly once. A stale or duplicate sequence performs no
second advancement and replays no protected query rows. Native refusal retains
the returned owner; a sequence refused before record admission is not consumed.

## How it executes

One create or resume command invokes at most one native advance. Supply explicit
nonzero `maximum_history_work` on each such command. Pending responses identify
native posture; another fresh command advances it. Fresh completion and recovered
receipt evidence are distinguished by `recovered`; recovery never invents a
`MoneyMovementResult`. Read uses the original retained World observation with
fresh query authority, `maximum_results` and `maximum_work`.

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

## How it relates to other features

Payment continuation describes a business payment; estate recovery owns external
effect custody; account-page cursors own query continuation. None is an authority
or replacement for this money operation. Money retained reads use native history,
not a current-view reconstruction from commit receipt counts.

## Inspection and debugging

Wire outcomes are `rejected`, `outcome_unknown`, `pending`, `refused`, `completed`,
`terminal`, `delivered` and `closed`. A close refusal does not discharge custody.
Duplicate live custody IDs and exhausted capacity reject before another effect.
A full command queue reports `saturated`; a closed worker reports `unavailable`.
`continuation.runtime()` lends the exact retained installation for independent
native catalog and cleanup access. It cannot reconstruct lost transport records.
Strict DTOs reject earlier protocol versions and fields belonging to a different
command. The native operation remains the authority for terminal and retry truth.

## Anti-patterns

Do not cache authenticated grants, reset a command budget in a hidden pump,
interpret queued timeout as not applied, or evict pending custody on a TTL.
Do not reconstruct the descriptor from a transport token or receipt.

Cold certification processes carry a named `trusted_root_pem` in their typed
`cold_certification` configuration. The generated courtroom CA signs the actual
identity server certificate; the client adds that root for its own TLS connection.
Missing or malformed roots refuse installation. The isolated Selenium container
imports that public root into its browser NSS database before sessions start;
administration HTTP receives the same named root explicitly. Browser sessions use
temporary profiles without certificate-bypass flags. Ordinary installation keeps
its normal TLS roots; certification never disables certificate or hostname validation.

## Current limits

This is in-process transport custody, without process-crash durability. Dropping
the final host continuation closes ingress and allows workers to end normally;
it performs no cleanup I/O and promises neither settlement nor native disposal.
Native World catalog records and physical cleanup obligations are distinct.
Independent native discovery applies only to the actual native records supported
by that catalog while their runtime remains alive, not to every ordinary posture.
Final runtime destruction has no added cleanup guarantee. A deliberately abandoned
ordinary descriptor cannot be recreated as equivalent operation custody.

## Related docs

See [the Bank consumer contract](public-consumer-contract.md) and the actual
[owning HTTP courts](../crates/bank-http-adapter/src/http/server/tests/mutation.rs).
