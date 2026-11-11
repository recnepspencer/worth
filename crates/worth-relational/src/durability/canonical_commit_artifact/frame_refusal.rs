use super::RelationalCanonicalCommitArtifactDigest;

/// Why bytes are not an intact canonical commit artifact frame. Every
/// variant refuses before the payload is decoded; none is retried or
/// reinterpreted under another format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalCanonicalCommitArtifactFrameRefusal {
    /// The bytes end before the fixed-width header does.
    TruncatedHeader {
        available: usize,
    },
    /// The leading bytes are not the artifact magic.
    UnrecognizedMagic,
    SizeLimit {
        declared_payload: u64,
        maximum_frame: usize,
    },
    /// The frame names an artifact format version this build does not read.
    UnsupportedVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    InvalidVersion,
    InvalidPayloadShape,
    /// Fewer payload bytes follow the header than it declares.
    TruncatedPayload {
        declared: u64,
        available: u64,
    },
    /// More bytes follow the header than the declared payload.
    TrailingBytes {
        declared: u64,
        available: u64,
    },
    /// The payload does not hash to the digest the header records.
    DigestMismatch {
        recorded: RelationalCanonicalCommitArtifactDigest,
        computed: RelationalCanonicalCommitArtifactDigest,
    },
}

impl std::fmt::Display for RelationalCanonicalCommitArtifactFrameRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TruncatedHeader { available } => write!(
                formatter,
                "canonical commit artifact header truncated at {available} bytes"
            ),
            Self::SizeLimit { declared_payload, maximum_frame } => write!(formatter, "canonical artifact payload {declared_payload} exceeds complete frame bound {maximum_frame}"),
            Self::UnrecognizedMagic => {
                formatter.write_str("bytes are not a canonical commit artifact")
            }
            Self::UnsupportedVersion(reason) => write!(formatter, "canonical commit artifact version {} is unsupported ({:?})", reason.produced().get(), reason.posture()),
            Self::InvalidVersion => formatter.write_str("canonical commit artifact version zero is invalid"),
            Self::InvalidPayloadShape => formatter.write_str("canonical v8 commitment fields are malformed"),
            Self::TruncatedPayload {
                declared,
                available,
            } => write!(
                formatter,
                "canonical commit artifact payload truncated: {available} of {declared} bytes"
            ),
            Self::TrailingBytes {
                declared,
                available,
            } => write!(
                formatter,
                "canonical commit artifact carries {available} payload bytes but declares {declared}"
            ),
            Self::DigestMismatch { recorded, computed } => write!(
                formatter,
                "canonical commit artifact payload digest {computed:?} differs from recorded {recorded:?}"
            ),
        }
    }
}

impl std::error::Error for RelationalCanonicalCommitArtifactFrameRefusal {}
