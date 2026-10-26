# Record-frame 17 vectors

Grammar17 adds an optional whole-orbit query slot and a distinct orbit disclosure
selector. The retained16 files are unchanged and must be refused.

The new fixed literals use payload-unchanged families: entity-only schema,
conditional binding, native aspect, operation contract, artifact, domain operation
and package-root.
Their independent derivation changes only each first big-endian u16 from16 to17.
The minimal archive contains only a package-root record; its manifest and payload
stay fixed. The envelope embeds that new archive and its recomputed SHA256,
retaining metadata, lengths and dummy signature bytes. It claims no real signature.
Current production encoders must match all new vectors exactly.

Query-bearing payloads are deliberately not derived by header substitution. The
execution declared-orbit court exports the actual authored schema, round-trips
its real slot and disclosure bytes through the current codec, and freshly
readmits the decoded declaration. The archive all-family court likewise exports
and reconstructs its real current records.
