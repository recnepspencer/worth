# Estate freeze recorded correction

## Boundary review

The ordinary freeze mutation resolves the exact estate/account relation, requires
an open account, and writes Frozen. Its aftermath currently names an inverse as
strings and captures Status; it has no executable typed correction binding.
The existing capability owner supplies active-grant, validity, account relation,
delegation, principal composition, and current estate workflow-stage checks.
Historical admission cannot replace those checks during correction.

The inverse restores the typed captured Status through a separate ordinary
mutation. It uses the same FreezeAccount grant meaning and account relationship
through an operation-bound capability declaration. It requires the current
account still to be Frozen, the exact current estate/account relation, and an
Open retained status. A revoked grant, changed workflow stage, changed relation,
or independently changed account status must refuse before publication.

## Implementation

1. Add `schema/estate/freeze_correction` children for the typed source association,
   inverse input/binding, operation declaration, and capability request. Replace
   only FreezeAccount's string inverse declaration with the typed mechanism.
2. Install the inverse's field/relation read and write set and reuse the existing
   FreezeAccount capability target, composition and currentness contract through
   a sibling operation-bound capability. Register the correction and mutation in
   the estate contribution. Register the ordinary provider and application action.
3. Add a handler in `mutation_handlers/restore_frozen_estate_account.rs` that reads
   the current estate/account and Frozen status before writing the retained Open
   status. Preserve ordinary native candidate and capability publication custody.
4. Extend the real estate fixture with correction courts for successful status
   restoration, original causal source, exact retry, independently moved status,
   current workflow-stage movement, and real capability revocation. Reuse existing
   ordinary wrong-account and capability boundary evidence. Coordinate the actual
   process transport court with the parent; do not claim process evidence from an
   in-process fixture.

## Ownership and verification

This slice owns the new estate children and narrow estate contribution, capability
installation, operation installation, handler module, provider and feature hunks.
Coordinate shared provider/composition changes with the workflow agent. Parent
owns HTTP, correction projection/custody and process orchestration. No private
files, independent Cargo commands or later-phase work. Use the coordinator's fast
metadata/native batch, and preserve adequate unchanged evidence. Independent
review must inspect fresh capability admission and the recorded-value derivation,
not merely the string posture or compilation.

## Process authority-change ownership addendum

The existing `BankRevocationCommand` already owns native capability publication,
fresh-principal validation, keyed recovery, cleanup and close. Query clones its
program runtime during capture. The remaining borrow is Bank's preparation
closure, which borrows the installation-minted invariant projection authority.
Store that same authority in an `Arc` in BankIdentityRuntime and capture its Arc
plus the existing program handle in revocation preparation. Extract the existing
projection/materialization helper so it borrows those retained owners. The public
constructor can then return a static command without retaining a request or a
grant, cloning an authority, or introducing a second executor.

The HTTP child stores Active/Completed/Terminal native revocation owners, delegates
advance, cleanup and close directly, and describes the original native receipt.
Every refusal retains its returned owner. Parent owns the existing serialized
worker's variant and wire registration. Source-completion observation is absent.
Expose native publication posture through the same existing posture mapping if
the capability command does not yet have a read-only posture accessor; never infer
it from a transport action counter. Independently revoke the freeze grant in the
real process journey and observe refusal to restore the still-frozen account.

## Authenticated process handoff

The HTTP mutation worker already serializes commands, reauthenticates each request,
retains native source owners before replying, and preserves refused close custody.
Its money-only names and fixed retained-delivery macro are the integration gap.
Freeze and restoration declare source completion, so giving them retained delivery
or the estate rail-handoff state would manufacture a different contract.

Extend this same worker with typed freeze and correction commands. Decode only
descriptive estate/account IDs and source coordinates; the installed capability
admission and exact historical lookup remain authoritative. Rename its local
money custody/record to source mutation custody/record, and parameterize its local
owner expansion with the action's concrete delivery mode. Retained actions use
their existing history-bounded advance; source-completion actions use ordinary
advance. Both preserve their original owners, evidence, cleanup and close.

Keep the macro in `mutation_custody/source_operation.rs`, the domain variants in
`mutation_custody.rs`, and estate correction construction in the existing correction
child. Add freeze/revocation grants to the explicit process estate seed. The process
court authenticates the specialist and independent account owner, freezes and
closes, restores and closes, then freezes a new occurrence and revokes the real
grant before attempting its correction. Ordinary account queries independently
observe Frozen/Open/Frozen and the foreign account remains unchanged.

Capability revocation uses its existing native capability command owner, whose
borrowed preparation needs an owned runtime handoff before it can be stored across
HTTP requests. Review that ownership boundary separately; no transport executor
may replace it or drop an unpublished/refused owner. Batch metadata after these
handoffs settle, then run the focused native and authenticated process courts.

### Preparation refusal classification

Review found that the existing estate HTTP classifier loses request interruption
inside invariant projection and candidate preparation. The new retained revocation
closure reaches those same owners. Correct the shared estate classifier at those
exact nested variants, forwarding the existing Relational execution and provider
session classifiers. Preserve current authority denials as authority denials.
Exercise cancellation/deadline twins through the new capability command adapter;
keep the original owner unchanged on every refusal. Other estate refusal meanings
and all execution policies stay with their current owners.
