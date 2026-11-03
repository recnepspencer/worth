# Atomic authored retirement custody

Public issue #39. D1044 (2026-11-02) accepts source-scope ownership as an explicit
installed cascade contract. This supersedes the D1043 custody block.

## Authority and ownership

`WorthQueryApplicationProducerBinding::OWNERSHIP` defaults to
`ProducerPublicationOwnership::Independent`. `SourceScopeCascade` declares that
complete producer publications are derived rows owned by their recorded source
scope. Query installs this declaration only for an input-field entity scope whose
source query selects that entity directly. Principal/shared/context traversal,
component/orbit and managed-index root selections refuse with a typed producer
subject. Every producer in one output family must agree. The policy and source
entity participate in installed declaration comparison and producer edition.

This uses the existing authenticated `source_scope`, already part of Native route
identity and derived from Query's actual operation scope. No caller roster, relation
name, dependency hash, field value or incidence establishes ownership. In
particular, two independent producers may read the same source. Owning a created
edge never adopts its external endpoint entity.

The installed authored correction binding supplies a distinguished source anchor:
`retire_created` fixes its original Create target; `restore_retired` fixes the
original Retire target and inverse Create role. Query validates the exact logical
key and physical source through the existing correction binding before handing
it to Native. The anchor remains separate from producer publication membership.
Native checks its exact current native generation, unique canonical allocation,
actual ordinary Create patch and absence from every required-output origin at birth.

## Closure and atomic publication

Native selects current publications on the selected linear ancestry, bounded by
the actual canonical catalog. It computes the complete live closure independently
of the submitted roster: opted-in publications scoped by the anchor, followed by
publications scoped by any live member already in the closure. The request pays
for ancestry, route, member, scope and proof comparisons through the existing
Execution work ceiling. There is no fresh allowance, new ceiling or global scan.
An incomplete/cyclic ancestry or unavailable custody refuses.

The roster must equal the anchor plus every live member of every selected
publication. Missing members/publications, extra or unrelated entities, repeated
targets, another source's publications and invalid authored anchors have typed
`AuthoredRetirementRefusal` causes and concrete `affected_records`. Tampered origin
or member records refuse against their exact canonical predecessor. Admission
seals the roster; publication cannot substitute a second caller roster. Native
also checks the actual deletion effects, including each owned edge, before the
single atomic transaction publishes.

A proof seal identifies both the commit and publication ordinal. Several routes
published in one atomic commit are distinct publications, even if producer text
or dependency bytes match. Each canonical `AuthoredRetirement` origin retains its
own producer, dependency, source route, implementation, birth and predecessor.
There is no merged synthetic producer.

## Restore and retained history

The exact recorded retirement prepares one Native inverse. The authored handler
creates the source with its declared logical identity; Native reconstructs every
owned publication's live predecessor payload and owned edges. One candidate
restores them all under fresh native generations. Owned relation endpoints are
remapped through the complete inverse correspondence. External references remain
external. Ordinary Native mutation admission generates endpoint aspects from the
fresh endpoints; stored endpoint aspects are not copied into a Create twice.

Query extends the original batch before its one complete-state validation. Native
checks the sealed inverse against the actual candidate before projecting application
touches. Only the inverse's own Creates are omitted from that projection; the source
and any additional application effects still require their installed touch contract.
The full mutation projection remains complete. This avoids validating a partial
restore or depending on later predicate-preparation infrastructure. The private
Native inverse authenticates each actual restored member and relation payload.
Every restored route has a separate canonical `AuthoredRestoration` origin and
keeps the original producing provenance. The old retired routes are fenced, so
one retirement cannot be restored twice. Cold readmission checks the complete
inverse roster and each member/relation against its own original publication.
Restoration is historical association; it does not manufacture computation facts,
Ready output, current READ/disclosure or a producer execution.

## Hand-worked courts

Ordinary: authored source S plus publication {R, C}, where R is a separate
realization record and C its cell: **3 live entities**.

Nested: S plus {Rp, Sc}, {Rc, Sl}, {Rl, C}: parent realization and child source;
child realization and leaf source; leaf realization and cell: **7 live entities**.
Each two-member publication owns its two source-association edges, so the retired
and restored edge rosters contain **2** and **6** edges respectively. The courts
state literal payload names and endpoint pairs, verify one committed candidate,
fresh generations, cold checkpoint restore, distinct per-route readmission,
precise refusal subjects, tampered carried records and a second-restore refusal.
A replacement court uses S plus {R, Cnew}, with Cold already retired by a prior
Preserve publication. Its hand-worked live roster is **3**, not 4: Cold is neither
deleted twice nor restored. Two independent publications sharing S and a publication
owned by a different ordinary source remain live. Existing single-publication warm/cold succession
courts retain their original behavior and assertions.

## Independent-path exclusion

An opted-in publication can retire only inside Native's admitted source cascade.
The old cold selector and the per-publication installer both refuse
`OwnedPublicationRequiresCascade`, carrying the exact publication commit and
source owner, with both records in `affected_records`. Partial anchorless rosters
receive the ownership refusal too. Anchorless retirement of a live owning scope
also refuses, whether the source has ordinary or independent-publication birth;
a mixed Create/Retire roster cannot hide that source deletion. The admission and
publication checks share one catalog-bounded, charged Native scope lookup. A producer-created intermediate source fails
the genuine-authored-birth check if offered as an anchor. Preserve succession may
replace a leaf but refuses deletion of a member that scopes a live owned
publication, using catalog-bounded ancestry and charged route/member comparisons.
The same source-severing check applies when an independent publication contains
an opted-in child's source. Independence does not sweep that publication into
the root's cascade and does not authorize leaving an owned child without its
source. Its complete legacy retirement still admits before the child exists;
after child publication, both retirement and replacement refuse with that child's
publication and source as subjects. The four-row court leaves S, independent Sc,
and owned {R,C} live after both refused candidates.
An owned publication also requires its source to remain live after a Birth or
Preserve candidate; restoration without that owner refuses
`PublicationOwnerUnavailable`. Independent producers keep their existing path.

The nested end-state court attempts intermediate retirement, then S + {Rp, Sc};
both refuse and all seven original entities remain live before the exact cascade.
The Query courts create S through an ordinary operation and demand real producer
publications. One pair reverses its Create through `retire_created`; another
retires through the declared ordinary Retire operation and reverses that receipt
through `restore_retired`. Reversing a correction receipt is the existing
reapplication contract, so the two declared correction associations have separate
end-to-end courts. Their literal key rosters are
{10, 101, 102} and {10, 101, 102, 1021, 1022, 10221, 10222}. An anchorless operation
reaches the legacy route and receives the typed ownership refusal, including
a partial roster and a mixed Create/Retire candidate. The retirement handler
observes only the distinguished source for its declared pre-image; Native owns
the derived publications' full historical payloads.

This custody contract is for required-output publication admission. It does not
change an ordinary unbound Native transaction's schema or CascadeDeletePolicy.

## Format and compatibility

The canonical commit artifact format is **2** (previously 1); Native checkpoint
magic is **WORTH-REL-CHK14** (previously CHK13). Canonical history now carries the
ownership marker, grouped retirement/restoration records, separate origins and
inverse correspondences. Authoritative digests, artifact projection and embedded
checkpoint encoding include these records. There are no durable installations
relying on the old format. This is a version bump with **no migration**; old framed
bytes are refused, never reinterpreted as declared ownership.

## Adjacent work

#35 supplies the source/realization split; references alone remain non-owning.
#37's existing single-publication codec transition and materialization behavior
is not redesigned here. A newly grouped authored publication cannot be collapsed
into its shared commit identifier for a codec transition; that case refuses until
it has distinct codec-transition custody. #34 and #36 remain outside this change.
The proprietary consumer tree is read-only and receives no edits.
