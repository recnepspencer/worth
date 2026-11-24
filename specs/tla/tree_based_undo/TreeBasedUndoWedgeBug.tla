----------------------- MODULE TreeBasedUndoWedgeBug -----------------------
(*
  The NoWedge regression: TreeBasedUndo with Phase 1's stale-forest wedge
  re-introduced (WedgeBug = TRUE). TLC must report NoWedge violated: an
  envelope stops while recording its output trees, .edit begins without
  releasing them, and neither its commit nor its discard can proceed.
*)
EXTENDS TreeBasedUndo
=============================================================================
