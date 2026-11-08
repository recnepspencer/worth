# Current ownership custody and durable readers

`Publication.custody` and `RoutePresence.custody` are optional durable fields.
An absent field retains the original publication's custody and re-encodes to
the identical legacy bytes. Explicit custody lets a successor advance its
origin without rewriting unchanged member/edge claims. Presence authenticates
explicit custody under `worth.current-route.v2`; legacy rows retain the v1
digest. These are row-grammar changes, even though canonical commit and
visibility commitments keep their existing meanings and bytes.

New native checkpoints emit `WORTH-REL-CHK20.1\n`. Their body grammar is CHK20,
with no transformed payload, member locator or commitment. A CHK20-only binary
refuses the new marker as the existing typed `UnsupportedLegacySemantics`
*before* it decodes the inline custody rows. The current decoder explicitly
readmits CHK20 and CHK20.1. Unknown revisions remain unsupported. The separate
paged-directory migration owns CHK21; this minor capability marker does not
implement or claim it.

Journal exports already use canonical artifact v7, which fences the public
pre-custody v6 reader with `UnsupportedVersion` before payload admission.
Neither canonical payload encoding nor its digest changes for this fix.
Previously written CHK20 checkpoints, including additive custody rows written
by the custody-aware slice-3 binary, remain readable by the current reader.
Old binaries cannot be retroactively changed to interpret already persisted
CHK20 custody bytes; the version fence applies to newly emitted bytes.

Completion prepares one live owned successor list per scope. Retirement,
catalog creation and advancement consume that same decision. All-retired
membership never preserves current custody; a live successor into a deleted
source scope refuses before effects are published. The pure owner policy court
covers the all-retired branch directly, independently of whether a particular
SDK producer workflow lowers an empty result to an absence event.

The publication payload still carries the complete origin. Making its bytes
flat in unchanged membership needs an immutable origin/reference representation
and its own format/validation cutover. Sharing the ownership writer's digest
with presence would remove their independent cross-check and is not that fix.
