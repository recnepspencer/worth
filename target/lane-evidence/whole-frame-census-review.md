# Independent census review

Source: dae78e6ff7; diagnostic-only work, no product changes.

First reviewer: `whole_frame_census_review`, REJECT proceeding on the original
census. Findings and resolutions:

1. HIGH, duplicate display/linkage identities. Use actual Rust PDB public
   linkage names before lossy display aliases; retain every distinct raw alias.
2. HIGH, generic stripping mistook function arrows for closing brackets.
   Both angle scanners now distinguish `->` from a generic delimiter.
3. HIGH, receiver-prefix ownership excluded a platform trait implementation
   for Option. Ownership now comes from Rust v0's defining impl path. The
   36264-byte ScopeKernel::invoke entry is included as worth_execution.
4. HIGH, incomplete anonymous attribution. PDB section contributions plus
   retained incremental COFF objects resolve all572 missing dev names. Module,
   section length, checksum and nonrelocation bytes agree. Every matching
   candidate has the same normalized owner path; no nearest-name guess.
5. MEDIUM, comparison scope. Recompute maxima using only the same3examples.
6. LOW, artifact identity. Assert PE RSDS GUID/age against PDB identity, and
   record both executable and PDB SHA-256 hashes.

Fresh reviewer: `whole_frame_corrected_census_review`.
**APPROVE — corrected diagnostic deliverable only.** No material findings.
It independently reproduced dev1658(375/485/798), dev3examples368(31/117/220),
release83named(0/11/72), release88unnamed physical entries. It checked all572
COFF owner matches, public-alias precedence, exact large-frame coverage in all10
artifacts, and executable/PDB hashes. Neither reviewer built or ran tests.

This does not approve a product fix or an implementation plan. The release
negative conclusion is established despite its explicitly recorded attribution
limit. Existing dev artifact source provenance is not claimed as a fresh build.

After review, the PDB command body was placed under a main guard so COFF
attribution can import its reader directly. The mechanical refactor changes no
decoder or matching rule; `python -m py_compile` passed for all5new main tools.
