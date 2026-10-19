//! Fixed native facts for the first incident edge outside the proven closure.
use super::{CodecRetirementClosureStop, OutputOrigin, RelationClosurePurpose};
use crate::identity::data::{EntityId, KindId, RelationId};
use std::fmt::{self, Display, Formatter};

pub(super) struct IncidentRelation {
    pub(super) id: RelationId,
    pub(super) kind: Option<KindId>,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
}

impl IncidentRelation {
    pub(super) fn outside_closure(
        self,
        purpose: RelationClosurePurpose,
        membership: (Option<usize>, Option<usize>),
        origins: &[OutputOrigin],
    ) -> CodecRetirementClosureStop {
        // Membership is the court's actual search result, not another probe.
        format!(
            "codec retirement has unsupported incident relation consequences outside its proven output closure; purpose={purpose}; relation={}:{}:{}; stored_kind={}; source={}; target={}; proven_member_count={}",
            self.id.partition_id.0,
            self.id.local_slot.0,
            self.id.generation.0,
            StoredKind(self.kind),
            Endpoint { id: self.source, ordinal: membership.0, origins },
            Endpoint { id: self.target, ordinal: membership.1, origins },
            origins.len(),
        )
        .into()
    }
}

impl Display for RelationClosurePurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CodecRetirement => "codec-retirement",
        })
    }
}

struct StoredKind(Option<KindId>);

impl Display for StoredKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(kind) => write!(f, "{}", kind.0),
            None => f.write_str("absent"),
        }
    }
}

struct Endpoint<'a> {
    id: EntityId,
    ordinal: Option<usize>,
    origins: &'a [OutputOrigin],
}

impl Display for Endpoint<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{} membership=",
            self.id.partition_id.0, self.id.local_slot.0, self.id.generation.0
        )?;
        match self.ordinal {
            Some(ordinal) => {
                let (_, kind, origin) = &self.origins[ordinal];
                write!(f, "inside(ordinal={ordinal},kind={},publication_commit={},publication_member_count={})", kind.0, origin.commit.commit_id.0, origin.members.len())
            }
            None => f.write_str("outside"),
        }
    }
}
