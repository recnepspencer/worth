# Milestone 10.7: Owner-Issued Shares And Close Custody

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires the database-foundation Milestone 3 Home close to have landed (it introduces
the close owners named below). Runs beside [10.1](milestone-10.1.md) and
[10.2](milestone-10.2.md). [10.3](milestone-10.3.md) Phase 3 consumes its census in
the close report.

Native runtime, Native medium and Store physical Serving handles are shared today as
public raw `Arc` clones. Close discovers that it is not the last holder only by
failing an exclusivity check, and it cannot say who the other holder is. Make every
share owner-issued and role-tagged, so retiring a holder is one typed transition and
a refusal names every live holder.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Native runtime and medium shares are owner-issued, role-tagged and non-cloneable. | Landed M3 Home close. | Phase 2 census; 10.8 funded issuance. | Freeze the share contract before callers migrate. |
| 2 | Close refusals carry a typed live-holder census; one retirement contract per role. | Phase 1. | 10.3 residual report. | Native first; Bridge and Query adapters after. |
| 3 | Bridge, Query and Store section adopt issued shares; raw escapes are deleted. | Phases 1–2. | Group closure; Store physical Serving successor. | One owner per slice. |

## Investigation

From the Milestone 3 integration (`codex/m3-host-home`, 2026-11-03):
- **Issuance escapes:**
  - Store `RelationalStoreSection` owns the serving runtime `Arc`, and `world_composition_runtime` returns a raw clone of it.
  - `RelationalRuntimeHandle` (`change_source/runtime_handle.rs`) accepts that raw `Arc` and derives `Clone`.
  - Bridge relational sources and Query provider/runtime handles retain those aliases, and Bridge source adapters are themselves `Arc`-shared.
- **Medium shares:** the canonical `DurabilitySubsystem` medium slot, the fork and publication service ports (derived `Clone`), transition steps, the transition history reader and publication preflight all retain medium shares. Checkpoint publication stores a `Weak`, so it must never count as a holder.
- **Refusals with no holder data:** Native `SharedRuntime` and `SharedBinding`, Store `SharedRuntimeCustody` and `PhysicalCustodyRetained`, and World `Busy` from `Arc::try_unwrap`/`get_mut`.
- **Scale:**

  | What | Count |
  |---|---|
  | `Arc` runtime/medium/native fields and parameters | about 350, in about 210 files |
  | Exclusivity checks | about 160 |
  | Retirement carrier types | 67 |
  | Retire/handoff functions | about 370 |
- **Cost:** retiring one holder touched 18 files. Discovering holders one rebuild at a time cost the integration 9 commits and 2.7 hours.

A Rust `Arc` records a count, not the identity of its holders, so no retrospective census is possible. The fix is at issuance.

## Phase 1: Owner-issued, role-tagged shares

The issuing owner (Native runtime, Native durability for the medium, and later Store section) hands out shares; callers never clone the underlying `Arc`.

```rust
pub struct OwnerShare<S: ShareSubject> { /* issuer handle, role, slot */ }

pub trait ShareSubject: sealed::ShareSubject {
    type Role: Copy + Eq + core::fmt::Debug; // closed enum, declared by the subject's owner
}

impl<S: ShareSubject> OwnerShare<S> {
    pub fn role(&self) -> S::Role;
    pub fn issue(&self, role: S::Role) -> Result<OwnerShare<S>, ShareIssueDenial>;
}
// No Clone. Drop releases the slot locally and never performs retirement work.
```

- **Role vocabulary:** each subject's roles are a closed enum declared at its owner. Shared role meaning, where two owners must agree on it, goes in `worth-foundational`.
- **Registry:** the issuer keeps the registry of live roles for its own subject. There is no global registry; the parent's prohibition on another global recovery ledger holds.
- **Affinity:** a share carries its issuer's typed Home/runtime identity, checked once at issuance. A foreign share is refused before any effect. There are no lifetime brands on public signatures.
- **Funding:** issuing a share that adds retained allocation consults the original request and exposes its footprint ([10.8](milestone-10.8.md)).
- **Observers are not holders:** a read observer receives an observation view with no retirement weight, reported separately.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| No raw escape | Attempt to clone a share or obtain the underlying `Arc` from a public boundary. | Compile failure; lawful `issue(role)` twin works. | Consolidated compile-contract lane. |
| Foreign share | Present a share from another Home's runtime to this Home's close. | Typed affinity refusal before effects; original custody unchanged. | Native/World owner court, focused. |
| Bounded issue | Issue and drop shares amid a large unrelated retained population. | Owner-local constant work; no scan. | Existing owner cost lane, scheduled only if expensive. |

## Phase 2: Census in refusals; one retirement contract per role

Every refusal that today means "another holder still exists" carries the live census:

```rust
pub struct HolderCensus<R> {
    pub live: SmallVec<[R; 4]>,
    pub stability: CensusStability, // Stable | ConcurrentlyChanging
}
```

- The census is diagnostic only. It never grants retirement authority, never auto-releases an external holder, and never adds a veto that the current gates do not have.
- It is taken without blocking the budgeted close path; a concurrently changing census says so.
- Each role implements one retirement contract: cooperative, funded by the original request, keeping custody on `Busy` or stop for a fresh retry, with reclaim separate from retirement. It replaces the bespoke per-holder carrier wherever that carrier only extracted a share.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Complete census | Hold distinct local and external shares, then close. | Refusal lists every live role; each disappears after its real release; close then succeeds. | Native and World Home close courts. |
| Unchanged gates | Repeat the Milestone 3 refusal, resume and physical-close journeys. | Same typed outcomes and custody as before; only the census is added. | Existing M3 close courts. |
| Retry | Stop the request between two role retirements, then resume with a fresh request. | Completed retirements stay done; the remaining role is retained, not re-run. | Native/World owner courts. |

## Phase 3: Callers adopt issued shares

Migrate one owner per slice:
1. Bridge relational sources and source adapters;
2. Query provider and runtime handles;
3. the Store relational section, deleting `world_composition_runtime`'s raw clone.

Each slice deletes its exclusivity checks and per-holder carriers as they become redundant.

**Store physical Serving** adopts the same primitive as a committed successor handoff in the Store roadmap. Its census is required before Store can claim a complete `PhysicalCustodyRetained` report.

Closure: adding a new holder of a covered subject means one role variant and one retirement implementation. A forgotten retirement is visible in the census, and a raw escape fails compilation. Record the peak-retained-memory change during close and recovery against the Milestone 3 baseline; it is measured, not promised.
