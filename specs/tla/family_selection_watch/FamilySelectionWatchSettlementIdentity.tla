---------------------- MODULE FamilySelectionWatchSettlementIdentity ----------------------
(*
  FamilySelectionWatch with certification comparing settlement identities
  (Mutant = CompareBySettlementIdentity). TLC must refute
  EqualAdoptionStaysCurrent: an equal adoption issues a new settlement with
  the same meaning, and the reader is moved and re-executes.
*)
EXTENDS FamilySelectionWatch
=============================================================================
