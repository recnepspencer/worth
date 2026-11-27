---------------------- MODULE FamilySelectionWatchImmediateMoved ----------------------
(*
  FamilySelectionWatch with the r5 rule (Mutant = ImmediateMovedOnRestore):
  an unwitnessed restore head at the coordinate answers Moved. TLC must
  refute Converges: the reader re-executes, the head stays unwitnessed, and
  the post-commit rebase re-drives it forever.
*)
EXTENDS FamilySelectionWatch
=============================================================================
