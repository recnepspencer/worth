---------------------- MODULE FamilySelectionWatchReadCoordinate ----------------------
(*
  FamilySelectionWatch with certification re-resolving at the read's
  coordinate (Mutant = ReresolveAtReadCoordinate). TLC must refute
  CleanOnlyFromReresolution after a fork: the reader's registration moves to
  the forked coordinate, the producer completes there, and re-resolution at
  the original coordinate still finds Missing.
*)
EXTENDS FamilySelectionWatch
=============================================================================
