use super::legal_home::LegalHome;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum DiagnosticCode {
    Bc1001IllegalCrateName,
    Bc1002UnreservedDomain,
    Bc2001BandDependencyViolation,
    Bc2002WorthToWorthyInversion,
    Bc3001DirectQueryEngine,
    Bc3002WrongQueryAudience,
    Bc3003QueryAudienceFacadeContract,
    Bc3101QuerySourcePath,
    Bc3102QueryPublicSignature,
    Bc3103QueryPublicReexport,
    Bc4001OrdinaryReplayImport,
    Bc5001RootOwnsRoad1Package,
    Bc5002SubworkspaceContractViolation,
    Bc5003SeedContractViolation,
    Bc5004HookAuthorityViolation,
    Bc6001LegacyReferenceGrowth,
    Bc6002LegacyReferenceBaseline,
    Bc7001AuthoritySealing,
    Bc7002LawSubstrateConfig,
    Bc7003SourceReachability,
    Bc8001SnapshotBaseline,
    Bc8002FacadeSnapshotDrift,
    Bc8003CrateDagSnapshotDrift,
}

impl DiagnosticCode {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 23] = [
        Self::Bc1001IllegalCrateName,
        Self::Bc1002UnreservedDomain,
        Self::Bc2001BandDependencyViolation,
        Self::Bc2002WorthToWorthyInversion,
        Self::Bc3001DirectQueryEngine,
        Self::Bc3002WrongQueryAudience,
        Self::Bc3003QueryAudienceFacadeContract,
        Self::Bc3101QuerySourcePath,
        Self::Bc3102QueryPublicSignature,
        Self::Bc3103QueryPublicReexport,
        Self::Bc4001OrdinaryReplayImport,
        Self::Bc5001RootOwnsRoad1Package,
        Self::Bc5002SubworkspaceContractViolation,
        Self::Bc5003SeedContractViolation,
        Self::Bc5004HookAuthorityViolation,
        Self::Bc6001LegacyReferenceGrowth,
        Self::Bc6002LegacyReferenceBaseline,
        Self::Bc7001AuthoritySealing,
        Self::Bc7002LawSubstrateConfig,
        Self::Bc7003SourceReachability,
        Self::Bc8001SnapshotBaseline,
        Self::Bc8002FacadeSnapshotDrift,
        Self::Bc8003CrateDagSnapshotDrift,
    ];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Bc1001IllegalCrateName => "BC1001_ILLEGAL_CRATE_NAME",
            Self::Bc1002UnreservedDomain => "BC1002_UNRESERVED_DOMAIN",
            Self::Bc2001BandDependencyViolation => "BC2001_BAND_DEPENDENCY_VIOLATION",
            Self::Bc2002WorthToWorthyInversion => "BC2002_WORTH_TO_WORTHY_INVERSION",
            Self::Bc3001DirectQueryEngine => "BC3001_DIRECT_QUERY_ENGINE",
            Self::Bc3002WrongQueryAudience => "BC3002_WRONG_QUERY_AUDIENCE",
            Self::Bc3003QueryAudienceFacadeContract => "BC3003_QUERY_AUDIENCE_FACADE_CONTRACT",
            Self::Bc3101QuerySourcePath => "BC3101_QUERY_SOURCE_PATH",
            Self::Bc3102QueryPublicSignature => "BC3102_QUERY_PUBLIC_SIGNATURE",
            Self::Bc3103QueryPublicReexport => "BC3103_QUERY_PUBLIC_REEXPORT",
            Self::Bc4001OrdinaryReplayImport => "BC4001_ORDINARY_REPLAY_IMPORT",
            Self::Bc5001RootOwnsRoad1Package => "BC5001_ROOT_OWNS_ROAD1_PACKAGE",
            Self::Bc5002SubworkspaceContractViolation => "BC5002_SUBWORKSPACE_CONTRACT_VIOLATION",
            Self::Bc5003SeedContractViolation => "BC5003_SEED_CONTRACT_VIOLATION",
            Self::Bc5004HookAuthorityViolation => "BC5004_HOOK_AUTHORITY_VIOLATION",
            Self::Bc6001LegacyReferenceGrowth => "BC6001_LEGACY_REFERENCE_GROWTH",
            Self::Bc6002LegacyReferenceBaseline => "BC6002_LEGACY_REFERENCE_BASELINE",
            Self::Bc7001AuthoritySealing => "BC7001_AUTHORITY_SEALING",
            Self::Bc7002LawSubstrateConfig => "BC7002_LAW_SUBSTRATE_CONFIG",
            Self::Bc7003SourceReachability => "BC7003_SOURCE_REACHABILITY",
            Self::Bc8001SnapshotBaseline => "BC8001_SNAPSHOT_BASELINE",
            Self::Bc8002FacadeSnapshotDrift => "BC8002_FACADE_SNAPSHOT_DRIFT",
            Self::Bc8003CrateDagSnapshotDrift => "BC8003_CRATE_DAG_SNAPSHOT_DRIFT",
        }
    }

    pub(super) fn default_legal_home(self) -> LegalHome {
        let pointer = match self {
            Self::Bc1001IllegalCrateName | Self::Bc1002UnreservedDomain => "tools/boundary-check/config/road1.toml [naming]",
            Self::Bc2001BandDependencyViolation | Self::Bc2002WorthToWorthyInversion => "tools/boundary-check/config/road1.toml [rule_contracts]",
            Self::Bc3001DirectQueryEngine | Self::Bc3002WrongQueryAudience | Self::Bc3101QuerySourcePath | Self::Bc3102QueryPublicSignature | Self::Bc3103QueryPublicReexport => "tools/boundary-check/config/road1.toml [rule_contracts.query_audience]; consume Query only through the audience facade configured for the crate band",
            Self::Bc3003QueryAudienceFacadeContract => "tools/boundary-check/config/road1.toml [rule_contracts.query_audience]; restore the configured workspaces/worth-query package or audience leaf",
            Self::Bc4001OrdinaryReplayImport => "tools/boundary-check/config/road1.toml [rule_contracts.reconstruction]; replay belongs in the cert band through worth-query-replay",
            Self::Bc5001RootOwnsRoad1Package => "tools/boundary-check/config/road1.toml [subworkspaces]",
            Self::Bc5002SubworkspaceContractViolation => "Cargo.toml [workspace] and tools/boundary-check/config/road1.toml [subworkspaces]; restore the declared workspace contract",
            Self::Bc5003SeedContractViolation => "tools/boundary-check/config/road1.toml [seed_skeletons]",
            Self::Bc5004HookAuthorityViolation => ".claude/settings.json [hooks.SessionStart, hooks.PostToolUse]; restore the canonical prepare and check commands",
            Self::Bc6001LegacyReferenceGrowth | Self::Bc6002LegacyReferenceBaseline => "tools/boundary-check/config/road1.toml [legacy_reference_ratchet]; shrink forbidden references or explicitly amend its configured snapshot and replacement guidance",
            Self::Bc7001AuthoritySealing | Self::Bc7002LawSubstrateConfig => "tools/boundary-check/config/road1.toml [law_substrates]; governed authority belongs to concrete worth-proof witnesses",
            Self::Bc7003SourceReachability => "tools/boundary-check/config/generated_source_exemptions.txt; every production Rust source must belong to a compiled target/module graph",
            Self::Bc8001SnapshotBaseline | Self::Bc8002FacadeSnapshotDrift | Self::Bc8003CrateDagSnapshotDrift => "tools/boundary-check/snapshots/; regenerate the governed snapshot explicitly with boundary-check --update-snapshots",
        };
        LegalHome::new(pointer).expect("diagnostic code legal homes are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{render_json, Diagnostic};

    #[test]
    fn every_default_diagnostic_names_an_existing_machine_artifact_first() {
        for code in DiagnosticCode::ALL {
            let diagnostic = Diagnostic::new(code, "arbitrary subject", "arbitrary prose");
            let json: serde_json::Value =
                serde_json::from_str(&render_json(std::slice::from_ref(&diagnostic)).unwrap())
                    .unwrap();
            let legal_home = code.default_legal_home();
            assert_eq!(
                json[0]["legal_home"],
                legal_home.as_str(),
                "{}",
                code.as_str()
            );
            assert_ne!(json[0]["legal_home"], "arbitrary subject");
            assert_ne!(json[0]["legal_home"], "arbitrary prose");
            let artifact = legal_home
                .as_str()
                .split_whitespace()
                .next()
                .unwrap()
                .trim_end_matches(';');
            let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            assert!(
                workspace.join(artifact).exists(),
                "{} names missing machine artifact {artifact}",
                code.as_str()
            );
        }
    }
}
