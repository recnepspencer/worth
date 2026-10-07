# Bounded transitive Query package identity

## Boundary review

A Query package retains complete typed application-schema declarations. Schema
meaning is freshly prepared by the declaration owner, and installation hashes
that canonical source under a finite entry/byte allowance. Package validation
currently copies every schema entry into another canonical sequence, prefixing
every key. That repeated representation can exhaust the package's 16 MiB ceiling
even when the admitted source and the genuinely additional package meaning fit.
Dropping declared members or accepting supplied digest claims would weaken meaning.

Only Query validation may derive package identity and grant package authority.
Portable reconstruction retains source records, freshly re-admits them, and
compares a freshly computed identity against independently selected expectations.
Neither descriptions nor supplied identity bytes grant installation authority.

## Implementation plan

Preserve complete schema source, the 16 MiB ceiling, aggregate entry preflight,
member validation, contract compilation and fresh portable reconstruction.
Change package canonicalization to v4: derive each schema's canonical digest
from its actual admitted basis, then include those fixed-width digests alongside
schema owner/name in the package basis. Schema digests and package digest must
share one finite entry/encoded-byte allowance. Report every child and parent
derivation, encoded byte, allocation and hash block in canonical work evidence;
normalize exhaustion diagnostics to the aggregate budget. No caller override,
claim-based shortcut, limit increase or compatibility identity is introduced.

The schema identity owner supplies the existing budgeted derivation internally.
A package identity schema-commitment module owns aggregate accounting. The
package identity module constructs the v4 parent basis. Keep production and
focused tests in files of at most 400 lines. Document the transitive commitment
and old identity rejection in the portable-package contract.

Acceptance covers a genuinely large declared schema admitted within the same
ceiling despite prefix-expansion overhead, complete child/parent work, exact
aggregate byte boundary and one-byte denial, multiple schemas sharing one
allowance, member drift and order independence, and fresh reconstruction with
forged/cross-spliced identity denials. Run the installation owner suite and
archive/reconstruction consumer checks, scoped formatting/line caps, boundary
and generated-context guards, and retained independent review. Consuming
applications verify their real installation before/after on the certified pin.
No unrelated Query currentness or geometry behavior belongs in this batch.

## Verification

Independent source/test review is clear. Installation's 322 tests and the
archive's 48 tests pass, including nine package identity controls and fresh
same-owner/name child-source splice rejection. Installation profile's two
execution tests pass. Formatting, whitespace, scoped Rust line caps and the
generated context guard pass. The boundary guard reports six
`BC7003_SOURCE_REACHABILITY` diagnostics in untouched `worth-ui-runtime`
replacement files; running the identical guard against original public
`901f52ef312609a3d1e9de80cd39556a6c0aa167` produces exactly the same six diagnostics.
No scoped source failure was introduced. Repairing that existing UI debt remains
with its owner.
