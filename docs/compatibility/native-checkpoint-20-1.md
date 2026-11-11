# Native checkpoint custody compatibility

CHK20.1 is the capability revision for successor custody rows and v2 route
presence commitments. Its body grammar and canonical payloads are unchanged.
The current decoder and inspector accept exactly CHK20 and CHK20.1. Unknown
frames, including future minor revisions, return the typed
`UnsupportedLegacySemantics` refusal before body decoding. Malformed bodies
remain `CorruptCheckpoint`.

There is a historical compatibility window: binaries that wrote successor
custody under the CHK20 header before `893f1e22d4` (including slice-3
`35da49b4c0`) could emit bytes that pre-custody readers misinterpret. The new
header cannot retroactively fence those files. Open those checkpoints with a
custody-aware reader and explicitly capture CHK20.1 before giving them to a
reader that might predate custody. The current reader deliberately retains
CHK20 admission; it does not infer absent custody from missing maintenance
state. The committed pre-custody CHK20 fixture and the synthetic custody-bearing
CHK20 twin exercise different parts of that compatibility contract.

A pre-revision inspector reports CHK20.1 as `CorruptCheckpoint`, while its
decoder reports unsupported semantics. Both refuse before reading custody;
this historical classification is not changed retroactively. Current inspection
uses the same frame owner as decoding and intentionally reports unsupported
semantics for an unknown frame.

Ownership completion also intentionally refuses a live owned successor into a
deleted scope at `PublicationStage::BundleAssembly` through `origin_denial`.
The outer type remains `TransactionCommitError::Publication`. Previously this
invalid proposal was refused later by corruption or roster integrity. Consumers
must preserve the typed publication refusal rather than depend on that later
cause or parse its text. Native policy and Query ownership courts cover the
current refusal; no consumer-specific recovery or retry is installed.
