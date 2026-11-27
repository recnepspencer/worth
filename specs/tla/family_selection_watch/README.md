# Family selection watch: TLA+ model

`FamilySelectionWatch.tla` models a `current_output` family selection read
(public #429). Its outcome is Missing, Unique or Ambiguous. The read is
recorded as a `FamilySelection` fact that every lineage head change in its
partition wakes. Certification re-resolves the selection at the certifier's
coordinate: Clean when the outcome is unchanged, otherwise Moved and
re-executed. The header documents the state and each mutant.

## Running

```sh
M=specs/tla/family_selection_watch
python tools/tla/run_tlc.py check $M/FamilySelectionWatch.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchKeyedClean.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchReadCoordinate.tla 4
python tools/tla/run_tlc.py check $M/FamilySelectionWatchSettlementIdentity.tla 4
```

The step bound ends every behaviour, so the configurations set
`CHECK_DEADLOCK FALSE`.

## Models

| Model | Expected | What it shows |
| --- | --- | --- |
| `FamilySelectionWatch` | exit 0 (61,617 distinct states, depth 12) | Every invariant holds, and `Converges` holds under weak fairness on `Certify`. |
| `FamilySelectionWatchKeyedClean` | exit 12, `CleanOnlyFromReresolution` | The pre-#429 rule: complete, fork, register, and a Missing reader certifies Clean while the lineage selects `e1`. |
| `FamilySelectionWatchReadCoordinate` | exit 12, `CleanOnlyFromReresolution` | Re-resolving at the read's coordinate breaks at a fork: fork, complete on the fork, register there, and a Missing reader certifies Clean. |
| `FamilySelectionWatchSettlementIdentity` | exit 12, `EqualAdoptionStaysCurrent` | Comparing the selected settlement's identity moves a Unique reader after an equal adoption. |

The trace validation of seed `0x7b02000f` against `TreeBasedUndoTrace.tla`
runs in S2. A trace module for this model needs production trace hooks and is
deferred to Phase 2E.
