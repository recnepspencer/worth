//! The roles of the two name schemes, declared once as Rust types. Each role
//! keeps the lowercase name earlier slices persisted, so a birth written
//! against the role name and one written against the role type are the same
//! birth.
use worth_query_decl::facade::application_naming::WorthQueryNameRole;

/// The roles a `Member` is born under.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
pub enum MemberRole {
    #[worth_query(name = "authored")]
    Authored,
    #[worth_query(name = "seam")]
    Seam,
    /// A piece may tell itself apart by a member or a seam authored beside
    /// it; the seam scheme sorts after this one.
    #[worth_query(name = "piece")]
    #[depends_on(Authored, Seam)]
    Piece,
    #[worth_query(name = "merged")]
    Merged,
    /// One repetition of a patterned member, told apart by its index.
    #[worth_query(name = "instance")]
    #[depends_on(Authored, Piece)]
    Instance(u32),
}

/// The roles a `Seam` is born under.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
pub enum SeamRole {
    #[worth_query(name = "authored")]
    Authored,
    /// A seam may tell itself apart by a member authored beside it.
    #[worth_query(name = "seam")]
    #[depends_on(Authored)]
    Seam,
    #[worth_query(name = "seam_piece")]
    SeamPiece,
}

/// The member roles every test group kind declares.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
pub enum GroupRole {
    /// The member a `Loop` follows (its anchor).
    #[worth_query(name = "host")]
    Host,
    /// The enclosing member a `Zone` follows (its boundary).
    #[worth_query(name = "wall")]
    Wall,
    /// The producing member a `Shell` follows.
    #[worth_query(name = "producer")]
    Producer,
    #[worth_query(name = "member")]
    Member,
}
