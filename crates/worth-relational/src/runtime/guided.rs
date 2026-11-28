use crate::history::HistoryAccess;
use crate::inspection::InspectionAccess;
use crate::publication::PublicationSurface;
use crate::replay::ReplayAccess;
use crate::runtime::durability_access::DurabilityAccess;
use crate::runtime::RelationalRuntime;
use crate::runtime::VisibilityReadContext;
use crate::simulation::{SimulationAccess, SimulationAuthority};
use crate::validation::InvariantAccess;
use crate::visibility::authority::VisibilityAuthority;
use crate::visibility::retention::VisibilityRetentionAuthority;

impl RelationalRuntime {
    pub fn snapshots(&self) -> VisibilityAuthority<'_> {
        self.visibility_authority()
    }

    pub fn read_truth(&self) -> VisibilityReadContext<'_> {
        self.visibility_reads()
    }

    pub(crate) fn visibility_reads(&self) -> VisibilityReadContext<'_> {
        VisibilityReadContext::new(self)
    }

    pub fn validation(&self) -> InvariantAccess<'_> {
        self.invariant_access()
    }

    pub fn compiled_artifacts(&self) -> SimulationAccess<'_> {
        self.simulation_access()
    }

    pub fn compiled_artifacts_authority(&self) -> SimulationAuthority<'_> {
        self.simulation_authority()
    }

    pub fn retention(&self) -> VisibilityRetentionAuthority<'_> {
        self.retention_authority()
    }

    pub fn inspect_what_happened(&self) -> InspectionAccess<'_> {
        self.inspection_access()
    }

    pub fn history(&self) -> HistoryAccess<'_> {
        self.history_access()
    }

    pub fn replay(&self) -> ReplayAccess<'_> {
        self.replay_access()
    }

    pub fn publication(&self) -> PublicationSurface<'_> {
        self.publication_access()
    }

    pub fn durability(&self) -> DurabilityAccess<'_> {
        self.durability_access()
    }
}
