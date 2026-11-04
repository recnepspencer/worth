//! The aggregate profile uses its own compiled schema and the same producer path.
#![expect(
    dead_code,
    reason = "Fault and checkpoint entry points are exercised by the ordinary profile; this profile shares its real producer fixture."
)]
const PRIVATE_AGGREGATES: bool = true;
producer_succession_fixture!();
#[path = "../producer_succession/aggregate_memberships.rs"]
mod aggregate_memberships;

#[path = "../producer_succession/predecessor_orbits.rs"]
mod predecessor_orbits;
