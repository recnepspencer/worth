//! The bounded-dimension rule contracts this schema publishes.
//!
//! Two contracts with distinct stable identifiers are installed side by side:
//! `bounded-dimension-v1` and `bounded-dimension-v2`. They are separate
//! identifiers rather than two versions of one identifier because the installed
//! invariant catalog is keyed by (identifier, execution point), so one host
//! cannot install two versions of the same rule contract at the same point.
//! The version suffix is spelled with a dash because the declaration facade
//! rejects a `.` inside an invariant identifier.
//!
//! `bounded-dimension-v3` is authored here but deliberately never installed, so
//! a program declaring it names a contract this host cannot support.

use std::num::NonZeroU64;

use worth_query_host::facade::declaration::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};

use super::BoundedDimensionSchema;

/// The part's dimension must not exceed ten.
pub struct BoundedDimensionV1;

/// The part's dimension must lie between five and twenty inclusive.
pub struct BoundedDimensionV2;

/// A rule contract no host in this court installs.
pub struct BoundedDimensionV3;

impl ApplicationInvariantMarkerIdentity<BoundedDimensionSchema> for BoundedDimensionV1 {
    const IDENTIFIER: &'static str = "bounded-dimension-v1";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<BoundedDimensionSchema> for BoundedDimensionV2 {
    const IDENTIFIER: &'static str = "bounded-dimension-v2";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<BoundedDimensionSchema> for BoundedDimensionV3 {
    const IDENTIFIER: &'static str = "bounded-dimension-v3";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

/// Bounded Relational work for this shared host's largest bootstrap and
/// workflow-publication candidates. The ordinary dimension mutation reserves
/// both installed rule ceilings explicitly, so this bound must also remain
/// within that operation's validator-work contract.
const FIRST_RULE_WORK_UNITS: u64 = 256;
const SECOND_RULE_WORK_UNITS: u64 = 256;

pub(super) fn first_definition(
) -> ApplicationInvariantDefinition<BoundedDimensionSchema, BoundedDimensionV1> {
    part_commit_boundary_definition(BoundedDimensionV1::reference(), FIRST_RULE_WORK_UNITS)
}

pub(super) fn second_definition(
) -> ApplicationInvariantDefinition<BoundedDimensionSchema, BoundedDimensionV2> {
    part_commit_boundary_definition(BoundedDimensionV2::reference(), SECOND_RULE_WORK_UNITS)
}

fn part_commit_boundary_definition<Invariant>(
    reference: worth_query_host::facade::declaration::application_schema::ApplicationInvariantRef<
        BoundedDimensionSchema,
        Invariant,
    >,
    work_units: u64,
) -> ApplicationInvariantDefinition<BoundedDimensionSchema, Invariant>
where
    Invariant: ApplicationInvariantMarkerIdentity<BoundedDimensionSchema>,
{
    ApplicationInvariantDefinition::new(
        reference,
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(
            NonZeroU64::new(work_units).expect("non-zero work budget"),
        ),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity("Part".to_owned())],
            [ApplicationInvariantScopeTarget::Entity("Part".to_owned())],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
