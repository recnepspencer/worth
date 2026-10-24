# Retained original package and current grammar expectations

`preactor_2bc_all_families.wqpkg` is the unmodified output of the production
`export_typed_records` and `encode_package_archive` at revision
`2bc38889c763853b8e56724b83deef8ca6caa71a` (record frame 6). It is 4,678 bytes;
SHA-256 is `a61843e3f022a3076851838a5ad984c6f04c67087d7867776fddc505bad48d5f`.
The existing `archive_stream/fixture.rs::all_family_package` supplied all twelve
families, including ApplicationSchema. The original test decoded the export,
reconstructed it and passed `validate_freshly` against its original package
identity before an optional hook wrote these exact bytes.

On 2026-10-15 the clean historical worktree
`worth-query-9.17.6.4-workflow-validation` ran exactly:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-package-archive --test archive_protocol --jobs 8 -- archive_stream::all_record_families_cross_one_archive_and_reenter_fresh_query_validation --exact --nocapture
```

Result: one test passed, zero failed, 75 filtered out; unoptimized test profile.
The sole temporary source hook, after the final fresh-validation assertion, was:

```rust,ignore
if let Some(path) = std::env::var_os("WORTH_ARTIFACT_PREDECESSOR_OUTPUT") {
    std::fs::write(path, &archive).expect("retain the validated historical production archive");
}
```

The task-specific output pointed outside Git. The exact hook patch and run log
are retained in the coordination directory. Removing only this hook left that
historical worktree clean at the same revision; its branch and stashes were not
changed. This run proves historical provenance, not current-reader acceptance.
The current refusal test consumes these original bytes without conversion. Its
positive counterpart is the existing all-family current export and fresh
validation test, still pending current execution.

The original `archive_v2.hex` and release-envelope golden remain unchanged.
New current expectations use independently specified record-frame version 8;
unchanged payloads and outer archive 2/manifest 1 framing retain their exact
bytes. The minimal archive has one record at offset 118. Its current counterpart
changes only that frame's two-byte version. The envelope 1 counterpart changes
only those nested archive bytes and their SHA-256 checksum at bytes 50–81.
Current expectations were constructed from this reviewed grammar without
calling a current production writer. The envelope decoder may still return the
original untrusted archive; the nested archive loader owns historical-frame refusal.

Frame8 removes the obsolete application-query dependency-ceiling payload. The
minimal archive and the two single-record goldens have no application query, so
their payloads are unchanged. Current v8 vectors were constructed from the
retained v7 vectors by changing the original two-byte record version; the
release envelope additionally updates only the nested archive checksum. The
v7 vectors remain historical bytes, alongside the original v6 vectors.

The combined record-frame9 grammar preserves explicit whole-aspect laws and
removes the application-query dependency-ceiling payload. Current v9 vectors
above contain no query, so their payloads preserve the whole-aspect v8 bytes;
only frame versions and the enclosing archive checksum change. Record8 remains
unsupported and is tested without payload reinterpretation. Outer archive2,
manifest1 and release-envelope1 remain unchanged.

Record12 appends the optional authored lifecycle family binding to a correction
association and its explicit completed-external reconciliation opt-in, following
record11's historical output-field manifest. The frozen
minimal, domain-operation, artifact, schema, native-aspect, conditional-binding
and package-root vectors contain no correction association. Their record12
expectations therefore change only each record's two-byte version. The minimal
archive's version changes at bytes 118–119. Its release-envelope counterpart
changes that nested archive and recalculates only its SHA-256 checksum at bytes
50–81. These expectations were constructed from the retained record10 vectors
with PowerShell byte operations and `SHA256.HashData`, independently of the
production encoder. The record10 archive and envelope remain refusal controls;
the protocol court also checks record11 rejection. The correction-binding
roundtrip court carries the actual nonempty authored family and logical key, plus
a completed-external procedure. That procedure's optional tag, text width and
bytes are checked against a handwritten wire suffix; its unknown optional tag
is separately refused. The frozen vectors remain unchanged because they contain
no correction association.

Record13 appends the optional retained-source reapplication binding identity to
the correction association. The frozen record12 payloads described above and
the complete operation-contract vector contain no correction association.
Their record13 copies therefore change only the record's two-byte version;
the release envelope additionally updates the nested archive's SHA-256 checksum
at bytes 50–81. PowerShell byte operations and `SHA256.HashData` constructed
these expectations independently of the production encoder. The record12
archive and envelope remain intact as refusal controls, and the protocol court
rejects record10, record11 and record12 before decoding any payload. The
correction-binding roundtrip carries the nonempty reapplication source identity.
