# Input-Selected Existing References

## What This Feature Is

A compiled operation body can replace one nullable outgoing relation with an
existing entity selected by an original input member. It does not allocate,
delete or transfer ownership of that entity.

## Why You Use It

Use it to assign a validator, select an existing parent, or clear an optional
reference without writing a custom handler that repeats native lookup logic.

## Stable Entry Points

`body.replace_relation_from_input(relation, identity_field, member, key)` is
available through `worth_query_decl::facade::application_schema`. Attach the
finished body to its operation definition and install the ordinary native body.
The relation supplies the admitted source scope; no dummy write is necessary.

## Core Mental Model

The input contains a business key, not graph authority. `Some(key)` requests
one existing identity through Query's indexed selection owner. Explicit `None`
clears the reference. Missing members, malformed values and unknown targets deny;
none of them means clear or keep. A repeated assignment to the same actual
endpoint emits no link/unlink churn. Sharing the target is allowed unless the
relation's own incoming constraints prohibit it.

All other native relation and application invariants still apply. Clearing can
be atomically refused, for example when it would violate a target's incoming
minimum; nullable source cardinality does not weaken that constraint.

The target field must be required, read-only, equality-queryable, unique,
unitless and frameless, with an installed readable identity scalar codec that
admits bounded native input. The source relation must have maximum one outgoing
endpoint and no positive minimum. A mandatory relation is not this nullable API.

## How It Executes

Query admits the original canonical member through the installed codec, observes
the complete bounded identity lookup and current adjacency, and retains the actual
old/new endpoints into candidate assembly. Replacement unlinks the old endpoint
and links the selected one in the same candidate. Clearing only unlinks.

Native publication revalidates retained facts and invariants. A competing
membership change can make preparation stale; it cannot silently retarget a
prepared candidate. Request authorization still belongs to the installed operation
and its policies. Successful lookup is not business eligibility or permission.

## Small Example

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Supply the declared schema and operation binding.
let body = AssignValidator::reference()
    .body()
    .replace_relation_from_input(
        Validator::reference(),
        PrincipalIdentity::reference(),
        "validator", // original Option-valued command member
        "validator", // candidate-local relation key
    )?
    .finish()?;
```

Declare source/target/identity/relation read capabilities and relation Link/Unlink
capabilities for the operation. The completed definition derives its exact read
and program contracts. Native publication grows the actual effect buffer
fallibly; suppliers do not forecast candidate link/unlink counts.

## Real Example

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Supply the declared schema and operation binding.
let body = AssignValidator::reference()
    .body()
    .assign_optional_input(Comment::reference(), "comment")?
    .replace_relation_from_input(
        Validator::reference(), PrincipalIdentity::reference(),
        "validator", "validator",
    )?
    .finish()?;
```

The comment and reference publish together. A same-target assignment preserves
the edge, but a changed comment can still supersede evidence that certifies it.
Add real eligibility and editability requirements using existing authored
requirements/capability policy, not a host lookup or an invented tenant restriction.

## How It Relates To Other Features

Signed graph subjects that traverse the reference participate in the existing
selection-scoped lifecycle. Replacement or clearing supersedes dependent evidence;
same-endpoint assignment alone preserves it. Independent subjects remain current,
and historical artifacts remain retained. A signing body cannot retarget its own
certified Subject path while claiming the old preimage is unchanged.

Use creation links when allocating a new entity. Use retiring replacement only
for an explicitly disposable predecessor; this API never deletes the target.

## Inspection And Debugging

Inspect `body.reference_inputs()` for portable correspondence. Query-produced
denials distinguish malformed input, unknown/ambiguous resolution and stale
publication; preserve those distinctions. Read the committed relation through a
native query rather than echoing the submitted key as proof of success.

Program/evolution meaning includes every reference component. New bodies use
canonical body profile 27 and portable archive profile 37. Bodies without reference
inputs retain their prior profiles; these profiles do not advance schema versions
or rewrite released package identity.

## Anti-Patterns

Do not resolve a key in the host and pass a raw entity ID as authority. Do not
unlink and link in separate commits, treat a missing member as `None`, delete the
old target, or maintain a separate signature invalidator.

## Current Limits

This profile replaces nullable outgoing single references, not collections,
mandatory references or reverse selections. The same relation cannot also be
owned by a creation link, retirement rule or another reference declaration in
that body. Target existence does not establish workflow approval or a signer's
authorization. Replay and recovery keep the binding's ordinary native posture.

## Related Docs

- [Scope-bound bodies](scope-field-operation-bodies.md)
- [Graph-certified subjects](graph-certified-subjects.md)
- [Retiring replacement](retiring-selection-replacement.md)
