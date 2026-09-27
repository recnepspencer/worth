# WORTH Signal Compatibility Transition Plan

## Purpose

DX cleanup often stalls when every change becomes hostage to vague breakage
anxiety.

This document exists to prevent that.

---

## Cleanup Principles

- use deprecation where it buys clarity
- use containment where immediate deletion is premature
- use immediate removal for clearly internal/certification-only surface
- prefer guided replacements over leaving deprecated ceremony as the only real
  path

---

## Migration Tools

Allowed migration strategies:

- deprecation ladders
- namespace containment
- feature-gated transitional exports
- codemod-able renames
- example and docs replacement of old flows

---

## Requirement

Major cleanup work in phases 3 through 5 should record migration logic here as
the work is designed, not only at the end.

---

## Default Compatibility Posture

Until publication shape is fully locked:

- preserve capability where possible
- change namespace prominence aggressively
- prefer guided replacements over preserving old ceremony as the blessed path
- remove clearly internal/certification-only surface without apology

---

## Preferred Migration Order

1. introduce the new guided or contained surface
2. move docs and examples to it
3. de-emphasize old raw flow
4. deprecate when useful
5. remove once the boundary is coherent enough

---

## Immediate-Removal Rule

The following classes should not receive a long deprecation ladder if they are
removed from the visible public boundary:

- harness/certification support
- internal architectural contract markers
- obvious support-only scaffolding

These are not part of the product promise.

---

## Containment Rule

When a surface is still real but too noisy for the main public path:

- move it into a specialist namespace first
- avoid breaking specialist capability unless there is a stronger redesign
  reason

This is especially relevant for:

- proof-bearing forms
- merge/reconciliation machinery
- lineage/history specialists
- reuse/equivalence machinery

---

## Condensation Rule

When replacing raw ceremony with guided surfaces:

- keep the raw capability available initially if it represents legitimate expert
  control
- do not keep the raw path as the primary documented path
- prefer migration through examples and docs first, then deprecation

---

## Rename Rule

If a public rename improves semantic clarity:

- prefer names that expose intent rather than internal implementation
- batch compatible renames together where possible
- make migration codemod-able when practical

---

## Release Gate For Deprecated Surface

No deprecated or transitional surface should remain indefinitely because of
cleanup anxiety.

Before publish, every deprecated or transitional area should have one of these
statuses:

- retained as legitimate specialist API
- removed from the main path but still intentionally contained
- fully removed
