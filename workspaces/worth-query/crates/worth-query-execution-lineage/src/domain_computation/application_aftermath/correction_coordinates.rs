//! Transportable source coordinates are resolved against actual completed native evidence.
use super::WorthQueryRecordedCorrectionSource;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::history::RelationalCommitReceipt;

/// Untrusted historical coordinates. Possession grants no disclosure or correction authority.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorthQueryCorrectionSourceCoordinates {
    world_owner: WorldOwnerCoordinate,
    world_commit: u64,
    relational: RelationalCommitReceipt,
    member: WorthQueryCorrectionSourceMember,
}

/// Which writer of one commit's coupled group a correction source names
/// (public #368). Every writer of an edit envelope shares the commit; the
/// envelope's root is `Primary`, and each later authored stage is named by its
/// framework stage identity.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub enum WorthQueryCorrectionSourceMember {
    Primary,
    AuthoredStage { ordinal: u64, identity: [u8; 32] },
}

impl WorthQueryCorrectionSourceMember {
    /// The member a writer's idempotency binding names.
    pub fn of_writer(
        stage: Option<&worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity>,
    ) -> Self {
        stage.map_or(Self::Primary, |stage| Self::AuthoredStage {
            ordinal: stage.ordinal() as u64,
            identity: *stage.identity(),
        })
    }
    /// Whether a writer with this stage identity is the named member.
    pub fn names_writer(
        &self,
        stage: Option<&worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity>,
    ) -> bool {
        *self == Self::of_writer(stage)
    }
    /// The canonical scalar text: `primary` or `stage:{ordinal}:{identity hex}`.
    pub fn text(&self) -> String {
        self.to_string()
    }
    /// Parses [`Self::text`]; anything else is not a member.
    pub fn parse(text: &str) -> Option<Self> {
        if text == "primary" {
            return Some(Self::Primary);
        }
        let (ordinal, hex) = text.strip_prefix("stage:")?.split_once(':')?;
        let canonical_ordinal = !ordinal.is_empty()
            && ordinal.bytes().all(|digit| digit.is_ascii_digit())
            && (ordinal == "0" || !ordinal.starts_with('0'));
        if hex.len() != 64 || !canonical_ordinal {
            return None;
        }
        let ordinal = ordinal.parse().ok()?;
        let nibble = |digit: u8| match digit {
            b'0'..=b'9' => Some(digit - b'0'),
            b'a'..=b'f' => Some(digit - b'a' + 10),
            _ => None,
        };
        let mut identity = [0_u8; 32];
        for (byte, pair) in identity.iter_mut().zip(hex.as_bytes().chunks(2)) {
            *byte = (nibble(pair[0])? << 4) | nibble(pair[1])?;
        }
        Some(Self::AuthoredStage { ordinal, identity })
    }
}

impl std::fmt::Display for WorthQueryCorrectionSourceMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => f.write_str("primary"),
            Self::AuthoredStage { ordinal, identity } => {
                write!(f, "stage:{ordinal}:")?;
                identity.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
            }
        }
    }
}

impl WorthQueryCorrectionSourceCoordinates {
    /// Carries descriptive fields received by an application transport.
    /// Selection resolves every field against retained native evidence.
    pub fn from_untrusted_fields(
        world_owner: WorldOwnerCoordinate,
        world_commit: u64,
        relational_branch: String,
        relational_commit: u64,
        relational_version: u64,
        relational_parents: Vec<u64>,
        member: WorthQueryCorrectionSourceMember,
    ) -> Self {
        use worth_relational::facade::{
            history::{BranchId, CommitId},
            identity::VersionId,
        };
        Self::from_untrusted_parts(
            world_owner,
            world_commit,
            RelationalCommitReceipt {
                branch_id: BranchId(relational_branch),
                commit_id: CommitId(relational_commit),
                version_id: VersionId(relational_version),
                parents: relational_parents.into_iter().map(CommitId).collect(),
            },
            member,
        )
    }

    pub fn from_untrusted_parts(
        world_owner: WorldOwnerCoordinate,
        world_commit: u64,
        relational: RelationalCommitReceipt,
        member: WorthQueryCorrectionSourceMember,
    ) -> Self {
        Self {
            world_owner,
            world_commit,
            relational,
            member,
        }
    }
    pub fn relational(&self) -> &RelationalCommitReceipt {
        &self.relational
    }
    pub const fn world_owner(&self) -> WorldOwnerCoordinate {
        self.world_owner
    }
    pub const fn world_commit(&self) -> u64 {
        self.world_commit
    }
    pub const fn member(&self) -> WorthQueryCorrectionSourceMember {
        self.member
    }
}

impl WorthQueryRecordedCorrectionSource {
    pub fn world_owner_coordinate(&self) -> WorldOwnerCoordinate {
        match self.world().owner_identity().durable() {
            Some(owner) => WorldOwnerCoordinate::Home { owner },
            None => WorldOwnerCoordinate::MemoryOnly {
                observation: self.world().owner_identity().get(),
            },
        }
    }
    pub fn coordinates(&self) -> WorthQueryCorrectionSourceCoordinates {
        WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
            self.world_owner_coordinate(),
            self.world().ordinal(),
            self.relational().clone(),
            self.member(),
        )
    }
}
