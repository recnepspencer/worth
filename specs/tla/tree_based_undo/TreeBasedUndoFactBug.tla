------------------------ MODULE TreeBasedUndoFactBug ------------------------
(*
  The FactsFollowLine regression: TreeBasedUndo with plan r1's defect
  re-introduced (FactBug = TRUE): navigation leaves the reversal facts alone.
  TLC must report FactsFollowLine violated: create, delete by correction,
  undo, and the "already reversed" fact is still active on the parent line,
  so a later delete on a sibling branch would refuse SourceAlreadyReversed.
*)
EXTENDS TreeBasedUndo
=============================================================================
