---------------------- MODULE FamilySelectionWatchKeyedClean ----------------------
(*
  FamilySelectionWatch with the pre-#429 rule (Mutant = FamilySelectionKeyedClean):
  wakes are keyed on the selected candidate, and a Missing reader is Clean by
  marks. TLC must refute CleanOnlyFromReresolution: a reader reads Missing,
  the producer completes, a demand certifies the reader Clean while the
  lineage selects e1.
*)
EXTENDS FamilySelectionWatch
=============================================================================
