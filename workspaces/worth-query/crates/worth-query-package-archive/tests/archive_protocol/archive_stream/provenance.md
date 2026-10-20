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
