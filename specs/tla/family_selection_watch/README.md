# Family selection watch: TLA+ model

`FamilySelectionWatch.tla` models a `current_output` family selection read
(public #429). Its outcome is Missing, Unique or Ambiguous. Under Option B the
read is recorded as a `FamilySelection` fact that marks never certify, so every
certification (a demand at any time, or a re-drive after a wake) re-resolves
the selection from the lineage heads at the certifier's coordinate. It is
Clean when the outcome is unchanged; otherwise it is Moved and the reader
executes again.

Wakes only drive convergence. A completion, readmission, recovery, equal
adoption or retirement may wake the reader's keys or escape the wake; a
restore never wakes. A reader a head change skips stays stale only until its
next demand, which still re-resolves. The header documents the state and each
mutant.

## Running

```sh
M=specs/tla/family_selection_watch
python tools/tla/run_tlc.py check $M/FamilySelectionWatch.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchKeyedClean.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchReadCoordinate.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchSettlementIdentity.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchImmediateMoved.tla 4
```

The step bound ends every behaviour, so the configurations set
`CHECK_DEADLOCK FALSE`.

## Models

| Model | Expected | What it shows |
| --- | --- | --- |
| `FamilySelectionWatch` | exit 0 (172,003 distinct states, depth 12) | `CleanOnlyFromReresolution`, `ServesCurrentSelection` and `EqualAdoptionStaysCurrent` hold, and `Converges` (`pending ~> ~pending`) holds under weak fairness on `Certify`. |
| `FamilySelectionWatchKeyedClean` | exit 12, `CleanOnlyFromReresolution` | The pre-#429 rule: a completion, then a demand on a Missing reader certifies Clean by marks while the lineage selects `e1`. |
| `FamilySelectionWatchReadCoordinate` | exit 12, `CleanOnlyFromReresolution` | Re-resolving at the read's coordinate breaks at a fork: fork, complete on the fork, register there, and a Missing reader certifies Clean. |
| `FamilySelectionWatchSettlementIdentity` | exit 12, `EqualAdoptionStaysCurrent` | Comparing the selected settlement's identity moves a Unique reader after an equal adoption. |
| `FamilySelectionWatchImmediateMoved` | exit 13, `Converges` | The r5 rule: any unwitnessed restore head answers Moved and leaves the re-drive pending, so a restore followed by a demand never converges. |

The trace validation of seed `0x7b02000f` against `TreeBasedUndoTrace.tla`
runs in S2. A trace module for this model needs production trace hooks and is
deferred to Phase 2E.
