# Round2 independent design review

Reviewer admitted_cleanup_layout_design_review: APPROVE, no blocking findings.
Read-only source review; no builds or edits. It confirmed all3 Core constructors
hold the actual original request and existing cleanup envelope reservation is
pre-effect. No public signature, proof, phase, protected owner or stack change.

1. Low: funding is Option<ExecutionMemoryReservation>, preserving native-only
None. Accepted; Core stores that exact optional ticket, never substitutes a grant.
2. Low: destruction order must keep funding until backing is gone. Accepted:
Core data precedes its ticket; managed state slot precedes return_owner; envelope
parts/reserved slot precede combined funding. Successful retry explicitly drops
the emptied state backing before releasing the return envelope.
3. Informational: genuine1MiB mains still decide H10; C2 is no waiver.

Implementation uses explicit data field access, no Deref or compatibility API.
Round1 Root6, Query3 and Store2 selected cases passed; ordinary consumer passed,
all3 example mains remain H10 red. Remaining Bank/consumer guard queue still runs
on immutable Source d2994fc364 before round2 edits.
