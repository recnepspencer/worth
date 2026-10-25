use super::RelationalBranchTransitionArtifactDigest;

/// Why bytes are not an intact branch transition artifact frame. Every
/// variant refuses before the payload is decoded; none is retried or
/// reinterpreted under another format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalBranchTransitionArtifactFrameRefusal {
    /// The bytes end before the fixed-width header does.
    TruncatedHeader { available: usize },
    /// The leading bytes are not the artifact magic.
    UnrecognizedMagic,
    /// The frame names an artifact format version this build does not read.
    UnsupportedVersion { found: u16, supported: u16 },
    /// Fewer payload bytes follow the header than it declares.
    TruncatedPayload { declared: u64, available: u64 },
    /// More bytes follow the header than the declared payload.
    TrailingBytes { declared: u64, available: u64 },
    /// The payload does not hash to the digest the header records.
    DigestMismatch {
        recorded: RelationalBranchTransitionArtifactDigest,
        computed: RelationalBranchTransitionArtifactDigest,
    },
}

impl std::fmt::Display for RelationalBranchTransitionArtifactFrameRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TruncatedHeader { available } => write!(
                formatter,
                "branch transition artifact header truncated at {available} bytes"
            ),
            Self::UnrecognizedMagic => {
                formatter.write_str("bytes are not a branch transition artifact")
            }
            Self::UnsupportedVersion { found, supported } => write!(
                formatter,
                "branch transition artifact version {found} is unsupported; this build reads {supported}"
            ),
            Self::TruncatedPayload {
                declared,
                available,
            } => write!(
                formatter,
                "branch transition artifact payload truncated: {available} of {declared} bytes"
            ),
            Self::TrailingBytes {
                declared,
                available,
            } => write!(
                formatter,
                "branch transition artifact carries {available} payload bytes but declares {declared}"
            ),
            Self::DigestMismatch { recorded, computed } => write!(
                formatter,
                "branch transition artifact payload digest {computed:?} differs from recorded {recorded:?}"
            ),
        }
    }
}

impl std::error::Error for RelationalBranchTransitionArtifactFrameRefusal {}
