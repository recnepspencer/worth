use crate::commit_strategies::data::CommitStrategyRegistration;
use crate::config::data::*;
use crate::config::data::{PlanningContract, RelationalExecutionModel};
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::DurabilityMode;
use crate::history::data::{BranchId, HistoryRetentionClass, VersionGraphPolicy};
use crate::schema::data::{
    runtime_descriptor_canonical_basis_policy, runtime_descriptor_semantics_policy,
    RelationalSchemaRegistry,
};
use crate::symbols::data::{ClientKeySymbolPolicy, StringInterner};
use crate::validation::data::InvariantCatalog;

pub(super) fn default_profile_config(profile: RelationalRuntimeProfile) -> RelationalRuntimeConfig {
    let base = |runtime_name: &str,
                diagnostics: RelationalDiagnosticsProfile,
                history_retention: HistoryRetentionClass,
                mvcc: MvccConfig,
                retention_policy: RetentionPolicy,
                storage_layout: StorageLayoutConfig,
                client_key_symbol_policy: ClientKeySymbolPolicy,
                visibility_cache_policy: VisibilityCachePolicy,
                durability: DurabilityPolicy,
                adjacency_policy: AdjacencyPolicy,
                cross_context_policy: CrossContextPolicy,
                cascade_delete_policy: CascadeDeletePolicy,
                publication: PublicationConfig,
                compiled_lane_policy: CompiledLanePolicy,
                relation_integrity_scope_budget: RelationIntegrityScopeBudget,
                initial_entity_capacity: usize,
                initial_relation_capacity: usize| RelationalRuntimeConfig {
        profile,
        execution: ExecutionConfig {
            runtime_name: runtime_name.to_string(),
            execution_model: RelationalExecutionModel::SingleLaneExecution,
            planning: PlanningContract::default(),
            compiled_lane_policy,
            relation_integrity_scope_budget,
        },
        diagnostics: DiagnosticsConfig {
            profile: diagnostics,
        },
        history: HistoryConfig {
            version_graph_policy: VersionGraphPolicy::CanonicalSerializedPublication,
            retention: history_retention,
            main_branch: BranchId("main".to_string()),
        },
        schema: SchemaConfig {
            registry: RelationalSchemaRegistry::default(),
            invariant_catalog: InvariantCatalog::default(),
            descriptor_semantics_policy: runtime_descriptor_semantics_policy(),
            descriptor_canonical_basis_policy: runtime_descriptor_canonical_basis_policy(),
        },
        commit_strategies: CommitStrategiesConfig {
            registrations: Vec::<CommitStrategyRegistration>::new(),
        },
        identity: IdentityConfig {
            client_key_symbol_policy,
            symbol_table: StringInterner::default().snapshot(),
        },
        storage: StorageConfig {
            initial_entity_capacity,
            initial_relation_capacity,
            mvcc,
            retention: retention_policy,
            layout: storage_layout,
            adjacency_policy,
            cross_context_policy,
            cascade_delete_policy,
        },
        visibility: VisibilityConfig {
            cache_policy: visibility_cache_policy,
        },
        publication: PublicationRuntimeConfig {
            policy: publication,
        },
        durability: DurabilityConfig { policy: durability },
        overrides: RelationalConfigOverride::default(),
        provenance: ConfigProvenance {
            profile,
            entries: Default::default(),
        },
    };

    match profile {
        RelationalRuntimeProfile::CertificationCore => base(
            "worth-relational",
            profile.default_diagnostics_profile(),
            HistoryRetentionClass::AuditGrade,
            MvccConfig {
                track_visibility_metadata: true,
                snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
                auto_reclaim_deleted_records: false,
                reclaim_batch_size: 128,
                retention_backend: RetentionBackend::PinTrackedRetention,
            },
            RetentionPolicy {
                backend: RetentionBackend::PinTrackedRetention,
                reclaim_batch_size: 128,
            },
            StorageLayoutConfig {
                entity_chunk_size: 1024,
                relation_chunk_size: 1024,
                scan_packet_size: 512,
            },
            ClientKeySymbolPolicy::PreferInterned,
            VisibilityCachePolicy {
                enabled: true,
                protect_branch_heads: true,
                protect_replay_retained: true,
                protect_active_snapshots: true,
                recent_version_window: 32,
            },
            DurabilityPolicy {
                mode: DurabilityMode::InMemoryCanonical,
                log: DurableLogPolicy {
                    retention_mode: DurableLogRetentionMode::RetainAllInMemory,
                    max_in_memory_envelopes: 4_096,
                    compact_after_checkpoint: false,
                },
                checkpoints: CheckpointPolicy {
                    compact_after_checkpoint: false,
                },
                store_layout: None,
            },
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
            CrossContextPolicy::SchemaControlled,
            CascadeDeletePolicy::CascadeDeleteRelations,
            PublicationConfig {
                coherent_publication_required: true,
                max_patch_records_per_commit: 4096,
                max_published_snapshot_handles: 256,
                max_active_snapshot_handles: 4_096,
                max_transaction_overlay_bytes: 268_435_456,
                max_transaction_footprint_loci: 262_144,
                max_transaction_savepoints: 4_096,
                max_prepared_candidates: 1_024,
                candidate_max_lifetime_millis: 30_000,
                max_prepared_root_bytes: 268_435_456,
            },
            CompiledLanePolicy::Disabled,
            RelationIntegrityScopeBudget {
                max_relation_kinds: 1_024,
                max_touched_entities: 8_192,
                max_deleted_entities: 4_096,
                max_scanned_relations: 65_536,
                max_planned_edges: 32_768,
            },
            64,
            64,
        ),
        RelationalRuntimeProfile::GeometryKernel => base(
            "worth-relational-geometry",
            profile.default_diagnostics_profile(),
            HistoryRetentionClass::AuditGrade,
            MvccConfig {
                track_visibility_metadata: true,
                snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
                auto_reclaim_deleted_records: false,
                reclaim_batch_size: 256,
                retention_backend: RetentionBackend::PinTrackedRetention,
            },
            RetentionPolicy {
                backend: RetentionBackend::PinTrackedRetention,
                reclaim_batch_size: 256,
            },
            StorageLayoutConfig {
                entity_chunk_size: 2048,
                relation_chunk_size: 2048,
                scan_packet_size: 1024,
            },
            ClientKeySymbolPolicy::PreferInterned,
            VisibilityCachePolicy {
                enabled: true,
                protect_branch_heads: true,
                protect_replay_retained: true,
                protect_active_snapshots: true,
                recent_version_window: 2,
            },
            DurabilityPolicy {
                mode: DurabilityMode::InMemoryCanonical,
                log: DurableLogPolicy {
                    retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
                    max_in_memory_envelopes: 2_048,
                    compact_after_checkpoint: true,
                },
                checkpoints: CheckpointPolicy {
                    compact_after_checkpoint: true,
                },
                store_layout: None,
            },
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 8,
            },
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::CascadeDeleteRelations,
            PublicationConfig {
                coherent_publication_required: true,
                max_patch_records_per_commit: 65_536,
                max_published_snapshot_handles: 64,
                max_active_snapshot_handles: 4_096,
                max_transaction_overlay_bytes: 67_108_864,
                max_transaction_footprint_loci: 131_072,
                max_transaction_savepoints: 1_024,
                max_prepared_candidates: 256,
                candidate_max_lifetime_millis: 30_000,
                max_prepared_root_bytes: 268_435_456,
            },
            CompiledLanePolicy::Disabled,
            RelationIntegrityScopeBudget {
                max_relation_kinds: 2_048,
                max_touched_entities: 32_768,
                max_deleted_entities: 8_192,
                max_scanned_relations: 131_072,
                max_planned_edges: 65_536,
            },
            256,
            256,
        ),
        RelationalRuntimeProfile::ChipSimulation => base(
            "worth-relational-chip",
            profile.default_diagnostics_profile(),
            HistoryRetentionClass::AuditGrade,
            MvccConfig {
                track_visibility_metadata: true,
                snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
                auto_reclaim_deleted_records: false,
                reclaim_batch_size: 512,
                retention_backend: RetentionBackend::EpochChunkRetention,
            },
            RetentionPolicy {
                backend: RetentionBackend::EpochChunkRetention,
                reclaim_batch_size: 512,
            },
            StorageLayoutConfig {
                entity_chunk_size: 4096,
                relation_chunk_size: 4096,
                scan_packet_size: 2048,
            },
            ClientKeySymbolPolicy::RequireInterned,
            VisibilityCachePolicy {
                enabled: true,
                protect_branch_heads: true,
                protect_replay_retained: true,
                protect_active_snapshots: true,
                recent_version_window: 2,
            },
            DurabilityPolicy {
                mode: DurabilityMode::InMemoryCanonical,
                log: DurableLogPolicy {
                    retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
                    max_in_memory_envelopes: 1_024,
                    compact_after_checkpoint: true,
                },
                checkpoints: CheckpointPolicy {
                    compact_after_checkpoint: true,
                },
                store_layout: None,
            },
            AdjacencyPolicy {
                backend: AdjacencyBackend::CompressedFanoutAdjacency,
                small_degree_inline_capacity: 8,
            },
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
            PublicationConfig {
                coherent_publication_required: true,
                max_patch_records_per_commit: 16384,
                max_published_snapshot_handles: 64,
                max_active_snapshot_handles: 4_096,
                max_transaction_overlay_bytes: 67_108_864,
                max_transaction_footprint_loci: 65_536,
                max_transaction_savepoints: 1_024,
                max_prepared_candidates: 256,
                candidate_max_lifetime_millis: 30_000,
                max_prepared_root_bytes: 1_073_741_824,
            },
            CompiledLanePolicy::DerivedCompiledLane,
            RelationIntegrityScopeBudget {
                max_relation_kinds: 4_096,
                max_touched_entities: 32_768,
                max_deleted_entities: 16_384,
                max_scanned_relations: 262_144,
                max_planned_edges: 131_072,
            },
            512,
            512,
        ),
        RelationalRuntimeProfile::AiWorkflow => base(
            "worth-relational-ai",
            profile.default_diagnostics_profile(),
            HistoryRetentionClass::Durable,
            MvccConfig {
                track_visibility_metadata: true,
                snapshot_release_policy: SnapshotReleasePolicy::ReleaseOnRetentionPass,
                auto_reclaim_deleted_records: true,
                reclaim_batch_size: 512,
                retention_backend: RetentionBackend::PinTrackedRetention,
            },
            RetentionPolicy {
                backend: RetentionBackend::PinTrackedRetention,
                reclaim_batch_size: 512,
            },
            StorageLayoutConfig {
                entity_chunk_size: 2048,
                relation_chunk_size: 1024,
                scan_packet_size: 1024,
            },
            ClientKeySymbolPolicy::PreferInterned,
            VisibilityCachePolicy {
                enabled: true,
                protect_branch_heads: true,
                protect_replay_retained: true,
                protect_active_snapshots: true,
                recent_version_window: 16,
            },
            DurabilityPolicy {
                mode: DurabilityMode::InMemoryCanonical,
                log: DurableLogPolicy {
                    retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
                    max_in_memory_envelopes: 1_024,
                    compact_after_checkpoint: true,
                },
                checkpoints: CheckpointPolicy {
                    compact_after_checkpoint: true,
                },
                store_layout: None,
            },
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::CascadeDeleteRelations,
            PublicationConfig {
                coherent_publication_required: true,
                max_patch_records_per_commit: 8192,
                max_published_snapshot_handles: 128,
                max_active_snapshot_handles: 4_096,
                max_transaction_overlay_bytes: 268_435_456,
                max_transaction_footprint_loci: 262_144,
                max_transaction_savepoints: 4_096,
                max_prepared_candidates: 1_024,
                candidate_max_lifetime_millis: 30_000,
                max_prepared_root_bytes: 268_435_456,
            },
            CompiledLanePolicy::Disabled,
            RelationIntegrityScopeBudget {
                max_relation_kinds: 1_024,
                max_touched_entities: 32_768,
                max_deleted_entities: 4_096,
                max_scanned_relations: 65_536,
                max_planned_edges: 32_768,
            },
            128,
            128,
        ),
    }
}
