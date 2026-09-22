# Bank World Product Contract

## Purpose

This contract freezes the legitimate bank world used to prove the ordinary
Query front door. It describes bank meaning, not transport behavior or Query
authority. Fixture labels, token claims, route-local checks, and test-only
mutation paths cannot change this contract.

## Domain model

The world contains:

- one or more institutions, each with explicit cash and settlement accounts;
- external-principal mappings keyed only by OIDC issuer and subject;
- bank principals, customer profiles, businesses, and institution-scoped
  employee assignments;
- personal and business accounts with an explicit currency and lifecycle
  status;
- account authorizations connecting a principal to an account through one
  declared customer role;
- payment intents and approval records;
- immutable journal entries containing immutable postings; and
- idempotency records bound to institution, principal, operation, and semantic
  payload.

Balances are derived from postings. No independently mutable balance field is
authoritative.

## Relationships and roles

Customer powers derive from current graph relationships:

- one external-principal mapping resolves through the primary graph to one
  principal row carrying its read-only typed `BankPrincipalId`; no parallel
  external-identity-to-bank-ID registry exists, while one principal may carry
  customer and employee relationships independently;
- a personal account has exactly one personal owner;
- a business has one or more owners and owns its business accounts;
- every account authorization joins one principal to one account with exactly
  one customer role: initiator, approver, or viewer;
- an initiator may prepare a payment but cannot satisfy a distinct-actor
  approval rule for that payment;
- every payment names one source account and one destination account; every
  approval names its payment and acting principal;
- a viewer can read only the account surfaces admitted by policy; and
- removing the relationship removes the corresponding power.

Employee powers derive from institution-scoped assignments:

- a teller may perform declared cash-facing operations for the assigned
  institution;
- an auditor may inspect declared institution activity but cannot move money;
- every employee assignment joins one principal to one institution with
  exactly one employee role; and
- customer and employee relationships remain independent when one principal
  holds both.

Every posting belongs to one journal entry and names one account. Every
institution-owned cash or settlement account is explicit; no operation implies
an unmodeled bank account.

No role string, fixture name, token claim, or route decision is authorization.

## Operations

The ordinary application must support these reads:

1. resolve the authenticated external principal to one active bank principal;
2. list accounts visible to that principal;
3. read account detail, current and available balance, authorized users, and
   activity;
4. list payments awaiting the principal's approval;
5. resolve a stable recipient principal for a direct personal transfer; and
6. subscribe to the same authorized account and activity results exposed by
   ordinary reads.

The mutation operation families are exact:

1. create a personal account;
2. create a business account;
3. apply explicit opening funding;
4. deposit through an institution cash or settlement account;
5. withdraw through an institution cash or settlement account;
6. transfer to a stable recipient principal rather than a fixture alias or
   caller-selected destination account;
7. initiate a business payment;
8. approve a pending business payment as a distinct authorized principal;
9. reject a pending business payment;
10. grant an account authorization;
11. revoke an account authorization; and
12. reverse a committed journal through a new typed correction operation.

Creation, opening funding, and reversal remain distinct intents even when an
adapter offers a composed user journey.

## Monetary invariants

- `Money<Currency>` is an exact, strictly positive requested movement amount.
- `SignedMoney<Currency>` is the distinct exact signed minor-unit value used
  by postings and derived balances; it cannot inhabit a movement input.
- Requested movement amounts are bounded by the exact minor-unit scalar.
- Every journal entry contains at least two postings and sums to zero in one
  currency.
- Deposit, withdrawal, transfer, opening funding, and reversal have distinct
  accounting purposes.
- Available-funds and account-status checks execute over proposed state.
- Postings, committed journal identity, and the semantic payload bound to an
  idempotency key are immutable.
- Reusing an idempotency key with a different semantic payload denies.
- Retrying the same committed intent returns the same semantic result without
  creating another journal entry.

## Outcome families

Reads distinguish delivered, absent, unauthorized, stale, cancelled, and
unavailable results. Mutations distinguish committed, approval required,
policy denied, invariant violated, stale, already committed idempotently,
aborted, cancelled, partial effect, and indeterminate. An adapter may map these
outcomes to transport vocabulary but may not collapse their meaning.

## Program evolution

The authoritative Bank host may support multiple validated Bank application
programs over the same installed native schema. Each product branch carries its
own selected program. Program evolution may add, change, or remove application
operations and rules, but it cannot reinterpret committed journal truth,
duplicate an external business effect, or turn a descriptive revision into
authority.

Adoption validates existing branch state under the target program and preserves
the exact disposition of continuations, resource reservations, and performed
effect recovery. A removed ordinary operation becomes unavailable on that
branch, while recovery of work performed under the source program remains
bound to its original occurrence and idempotency/outbox evidence. Only a
performed composite publication changes the selected program; sibling branches
may remain on different supported revisions.

## Ownership

- `bank-domain` owns every concept and invariant in this document.
- Query owns typed declaration, installation, authorization composition,
  touched-graph admission, branch-program adoption, execution progression, and
  result meaning.
- Relational owns graph facts and touched-graph proof.
- Signal owns policy evaluation evidence.
- The runtime bridge owns installed correspondence and lowering.
- HTTP and Authentik adapters translate protocol data only.
