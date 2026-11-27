---------------------- MODULE FamilySelectionWatchKeyedClean ----------------------
(*
  FamilySelectionWatch with the pre-#429 rule (Mutant = FamilySelectionKeyedClean):
  wakes are keyed on the selected candidate's settlement, and a Missing reader
  certifies Clean with nothing to compare. TLC must refute
  CleanOnlyFromReresolution: a reader reads Missing, the producer completes,
  a registration demands a certification, and the reader certifies Clean
  while the lineage selects e1.
*)
EXTENDS FamilySelectionWatch
=============================================================================
