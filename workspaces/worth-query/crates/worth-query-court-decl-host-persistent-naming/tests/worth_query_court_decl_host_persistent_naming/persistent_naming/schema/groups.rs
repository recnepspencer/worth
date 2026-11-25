//! The test group kinds: one per continuity rule, all over `Member` names.
use worth_query_decl::facade::application_naming::ApplicationGroupKind;

/// Follows its host member (`Anchor` continuity).
#[derive(Clone, Copy, Debug)]
pub struct Loop;
impl ApplicationGroupKind for Loop {
    const NAME: &'static str = "Loop";
}

/// Follows its walls (`Boundary` continuity).
#[derive(Clone, Copy, Debug)]
pub struct Zone;
impl ApplicationGroupKind for Zone {
    const NAME: &'static str = "Zone";
}

/// Follows its producing member (`ProducerRole` continuity).
#[derive(Clone, Copy, Debug)]
pub struct Shell;
impl ApplicationGroupKind for Shell {
    const NAME: &'static str = "Shell";
}

pub const LOOP_SCHEME: &str = "worth.tests.persistent-naming.loop.v1";
pub const ZONE_SCHEME: &str = "worth.tests.persistent-naming.zone.v1";
pub const SHELL_SCHEME: &str = "worth.tests.persistent-naming.shell.v1";
