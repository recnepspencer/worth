//! Test doors exposing the existing two halves of `checkpoint` unchanged.
//!
//! `DurabilityAuthority::checkpoint` captures one recovery basis, then
//! finalizes it. These doors let one exact certification court complete an
//! older capture after a newer checkpoint compacted its transitions. They add
//! no state, ordering or semantics: each calls the same private half that
//! `checkpoint` calls. No production code reaches them; their caller row is
//! in `tools/boundary-check/config/control-callers.txt`.
use super::checkpoint_capture::CapturedCheckpointBasis;

/// One captured, not yet finalized recovery basis from the capture door.
#[doc(hidden)]
pub struct CapturedRelationalCheckpointBasisForTest(CapturedCheckpointBasis);

mod owner_doors;
