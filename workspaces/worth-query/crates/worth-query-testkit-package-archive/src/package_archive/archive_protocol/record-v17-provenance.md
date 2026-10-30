# Record-frame 17 and 18 vectors

Grammar17 adds an optional whole-orbit query slot and a distinct orbit disclosure
selector. The retained16 files are unchanged and must be refused.

The new fixed literals use payload-unchanged families: entity-only schema,
conditional binding, native aspect, operation contract, artifact, domain operation
and package-root.
Their independent derivation changes only each first big-endian u16 from16 to17.
The minimal archive contains only a package-root record; its manifest and payload
stay fixed. The envelope embeds that new archive and its recomputed SHA256,
retaining metadata, lengths and dummy signature bytes. It claims no real signature.
Those exact record17 bytes remain frozen and are now refused by the current-only
record18 reader.

Query-bearing payloads are deliberately not derived by header substitution. The
execution declared-orbit court exports the actual authored schema, round-trips
its real slot and disclosure bytes through the current codec, and freshly
readmits the decoded declaration. The archive all-family court likewise exports
and reconstructs its real current records.

Grammar18 adds an explicit optional source-inspection association to mutation
descriptions. The record18 fixed vectors use only the same payload-unchanged
families listed above: each complete frame's initial big-endian u16 changes from17
to18, with its declared payload length checked. The minimal archive contains one
such package-root frame at byte118. Its manifest and payload remain unchanged.
The record18 envelope embeds that archive and its independently recomputed SHA256;
outer framing, metadata, lengths and dummy signature remain unchanged. Current
production encoders must match these vectors exactly.

Mutation descriptions change payload even when the association is absent: an
explicit option tag is required. No mutation-bearing schema vector is derived by
header substitution. The existing Execution optional-output court exports a real
non-correctable binding with authored source disclosure, encodes and decodes its
current schema, and freshly validates it. The archive mutation-description family
also tests every association field, malformed option tags and truncation.
