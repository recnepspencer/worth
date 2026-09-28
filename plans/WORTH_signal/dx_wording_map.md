# WORTH Signal DX Wording Map

## Purpose

This document maps the current formal surface to a friendlier guided vocabulary
without erasing precise underlying semantics.

Rule:

- guided surface can be more casual and natural
- raw surface stays formal where exactness matters

---

## Preferred Guided Wording

| Formal | Guided | Scope |
| --- | --- | --- |
| consume | use | docs / guided APIs |
| configuration | setup | docs / guided APIs |
| mutation | change | docs / guided APIs |
| invalidate | update / mark changed | guided APIs where semantics fit |
| validate | check | docs / lighter helper APIs |
| artifacts | details | docs where forensic precision is not required |
| lineage | history | guided entrypoints |
| execution | run / work | higher-level docs |
| dirty batch | batch change / batch update | guided API surface |
| explanation | explain / why | guided diagnostics entry |
| comparison | compare | guided diagnostics entry |

---

## Initial Surface Mappings

These are the low-risk aliases we can support without deep refactors:

- `mark_dirty` -> `mark_changed`
- `mark_dirty_with_regions` -> `mark_changed_with_regions`
- `DirtyBatch` -> `BatchChange`
- `SemanticBatchCommit` -> `BatchChangeResult`
- `facade::history` as the guided state-history namespace
- diagnostics `why(...)` as a guided alias for explanation access

---

## Guardrails

- do not rename formal core semantics where precision matters
- do not hide the formal terms from specialists
- do let the guided surface speak more naturally
