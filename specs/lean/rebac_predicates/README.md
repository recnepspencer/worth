# ReBAC path predicate proofs (Wave 9A-1)

`RebacPredicates.lean` models how WORTH-Query lowers authored ReBAC
conditions (`worth-query-declaration/src/application_schema/authorization_policy/`)
to the relational kernel's predicates and field constraints
(`worth-relational-mutation/src/authorization/constraint.rs`), and proves the
lowering means what the author wrote.

## Run

```
python3 tools/lean/run_lean.py check specs/lean/rebac_predicates/RebacPredicates.lean
```

Expected: `Lean kernel audit PASSED`. No `sorry`, no axiom outside `propext`,
`Classical.choice` and `Quot.sound`. Core Lean only; outside every Cargo
compile closure.

## Theorems

| Theorem | Property |
|---|---|
| `eq_sound` | `eq v` holds exactly on `some v` |
| `ne_sound` | `ne v`, lowered to `<` ∪ `>`, holds exactly on a present, same-family value other than `v`, on every family |
| `oneOf_sound` | `one_of vs`, lowered to one `Equal` path per value, holds exactly on a listed value |
| `absent_never_holds` | an absent field satisfies no condition (fail closed) |
| `other_family_never_holds` | a value of another scalar family satisfies no comparison |
| `deny_on_absent_field_is_inert` | a deny condition over an absent field never fires, which is why deny conditions on optional fields are refused |
| `ne_actor_sound` / `eq_actor_sound` | principal-field constraints compare two present, same-family values |
| `admitted_families_are_faithful` | integer, date, time and timestamp keys order as their domain values |
| `atMost_means_le` | on those families kernel `AtMost` is the domain `≤` |
| `decimal_text_order_is_not_domain_order` / `decimal_text_is_not_faithful` | decimal text orders "10000" before "9", so ordered comparisons are not offered on it |
| `expand_iff` | the cartesian expansion of alternatives has a satisfied path exactly when every condition has a satisfied alternative |
| `lowered_path_sound` | the installed paths of one authored path admit exactly when every authored condition holds |
