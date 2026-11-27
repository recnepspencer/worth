//! One real data image for observing a retained or freshly sealed fact.
use worth_relational::facade::{
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime,
        VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
};

#[derive(Clone, Copy)]
pub enum FactObservationBasis<'a> {
    Committed {
        runtime: &'a RelationalRuntime,
        snapshot: &'a SnapshotHandle,
    },
    Applied(&'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>),
}
impl FactObservationBasis<'_> {
    pub fn projection(self) -> Option<VisibilityProjectionView> {
        match self {
            Self::Committed { runtime, snapshot } => {
                runtime.read_truth().project_snapshot(snapshot)
            }
            Self::Applied(pin) => Some(pin.project_candidate()),
        }
    }
}

impl FactObservationBasis<'_> {
    /// A persistent group read re-observed at a committed basis; an applied
    /// image holds no published group state, so it refuses there.
    pub fn observe_group(
        self,
        read: &crate::domain_computation::primary_graph::application_attempt::fact::persistent_group::PersistentGroupRead,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >{
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Denial;
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact;
        let Self::Committed { runtime, snapshot } = self else {
            return Err(Denial::new(
                MissingAuthoritativeFact,
                "persistent group at an applied image",
            ));
        };
        let answer = read
            .observe_at(runtime, snapshot)
            .map_err(|_| Denial::new(MissingAuthoritativeFact, read.group_kind.clone()))?;
        Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::PersistentGroup {
            read: read.clone(),
            answer,
        })
    }

    /// A name-history read re-observed at a committed basis; an applied
    /// image holds no published name index of its own, so it refuses there.
    pub fn observe_name_history(
        self,
        read: &crate::domain_computation::primary_graph::application_attempt::fact::persistent_name_history::PersistentNameHistoryRead,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >{
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Denial;
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact;
        let Self::Committed { runtime, snapshot } = self else {
            return Err(Denial::new(
                MissingAuthoritativeFact,
                "persistent name history at an applied image",
            ));
        };
        let answer = read
            .observe_at(runtime, snapshot)
            .map_err(|_| Denial::new(MissingAuthoritativeFact, read.kind.clone()))?;
        Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::PersistentNameHistory {
            read: read.clone(),
            answer,
        })
    }
}
