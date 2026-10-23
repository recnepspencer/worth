# Bank recorded-source compensation

## Boundary review

Phase 2 requires the actual Bank accounting and process journey. The verified
source-output projector supplies original values from a declared, exactly-one
created role. SendMoney currently discards the native Journal returned by
`mutation_handlers/journal.rs` and declares no outputs. Its input contains
neither the generated Journal identity nor its institution.

The approved proposal snapshot obtains account institution membership through
the guarded InstitutionAccount projection. The Journal's authoring context is
the first posting's account, already used by `author_journal`. Retain that
context's institution as an immutable Journal field. This describes the context;
it does not assert that every posting belongs to that institution. ReverseJournal
continues checking all posting accounts and current authority independently.

The typed SendMoney correction must replace its no-aftermath declaration, bind
the actual created Journal, disclose under AccountSummary's current ViewAccount
contract, and derive the existing ReverseJournal input from original Journal
fields. Domain journals, current balances and JournalReversal remain the business
truth; no second accounting engine or compensation snapshot is introduced.

Correction's wire key is currently converted through an infallible declaration
method, whereas BankIdempotencyKey construction can refuse malformed input.
Propagate this through the existing ApplicationMutationIdentityDenial::Key and
ApplicationValueEncodeDenial::Validation channel before inverse construction or
candidate contact, preserving the ordinary key's validation and replay meaning.

Transport custody holds a native completed receipt but deletes its record on
normal close. Its identifier is not historical authority. The subsequent
transport handoff must select exact retained native completed evidence through
the existing owner, then freshly disclose/admit. The existing independent
reviewer confirmed that WorthQueryRecordedCorrectionSource already represents
World identity plus the full Relational receipt. Issue these coordinates from
the completed receipt; add select_recorded beside receipt selection, resolving
the original provider evidence and comparing its actual World identity. Both
entrances join one private selection core for correctability, ancestry and fresh
disclosure. Receipt selection retains its runtime and publication checks.
Missing or retired evidence refuses. No wire-reconstructed application receipt,
performed_by_world fallback, receipt registry or restart claim is introduced.

## Implementation order and proof

1. Add JournalInstitutionContext to the JournalIdentity contract and update its
   revision, declaration, shared money authoring grants, and bootstrap seeds.
   Populate it from the actual approved context account. Missing bootstrap
   context leaves the required field absent for native installation refusal.
2. Declare SendMoneyOutputs/SendMoneyJournal and bind the actual created entity.
   Install one SendMoney correction association and its two finite original
   output fields. Keep the ordinary ReverseJournal binding and handler.
3. Make inverse key conversion fallible at its existing declaration/preparation
   boundary, cut over every implementation, and cover invalid conversion and a
   lawful twin. Do not panic, truncate keys, or increase limits.
4. Exercise the real Bank runtime with two institutions: original movement,
   intervening movement, source-selected compensation, ordinary balances and
   activity, one balanced linked reversal, exact replay and current-authority
   refusal. Original-source Journal identity must differ from the intervening
   Journal. A hardcoded institution must fail the second context.
5. Resolve and implement the closed-custody exact-selection transport handoff,
   then extend TransportProcessWorld using independently authenticated nodes.
   Prove normal close, source selection, response-loss replay and hostile caller
   refusal. Review its owner boundary before introducing the API.
6. Reuse unchanged source-output, compatibility and recovery evidence. Batch
   affected metadata before native runtime/process codegen; use existing target
   caches, unoptimized incremental builds, unrestricted jobs and 64 MiB stacks.
   Run focused courts, formatting, dirty line caps, boundary-check and context
   enforcement. Reuse the existing source and evidence reviewers.

The Bank declaration lives in schema/correction.rs and its output role in
schema/operations/send_money_outputs.rs. Shared Journal authoring retains its
existing owner. Consumer documentation will show the normal source-selection
and correction entry, with detailed current examples in the existing guides.

Phase 2 closes only after its complete acceptance evidence passes. Phases 3–4
remain held. All work is public; private repositories are untouched.
