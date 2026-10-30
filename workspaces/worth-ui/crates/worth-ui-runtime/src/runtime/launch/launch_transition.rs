use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::allocation_frame_dispatch::UiAllocationFrameFrameworkScheduler;
use crate::runtime::allocation_receipt::UiAllocationReceiptLedger;
use crate::runtime::planning::allocation_planning::WorthUiRetainedAllocationPlanningEvidenceRegistry;

use super::build_active_state::build_active_runtime_state;
use super::launch_request::{
    WorthUiRuntimeLaunch, WorthUiRuntimeLaunchAuthority, WorthUiRuntimeLaunchDenial,
};
use super::preservation::WorthUiLastValidRuntimeState;
use super::runtime_instance::WorthUiRuntime;
use super::seal_artifact::seal_launch_artifact;

impl WorthUiRuntime {
    pub(crate) fn launch_prepared(
        admission: crate::facade::prepared_application_authority::WorthUiPreparedLaunchAdmission,
        retained_allocation_planning_evidence: Rc<
            WorthUiRetainedAllocationPlanningEvidenceRegistry,
        >,
    ) -> Result<(Self, crate::facade::WorthUiHostSessionAuthority), WorthUiRuntimeLaunchDenial>
    {
        let crate::facade::prepared_application_authority::WorthUiPreparedLaunchAdmission {
            lowering_authority,
            initial_allocation_commit,
            artifact,
            artifact_digest,
            snapshot_digest,
            diagnostic_policy,
            query_binding,
            host_session_plan,
            change_profile,
        } = admission;
        let mut host_session =
            crate::facade::WorthUiHostSessionAuthority::activate(&host_session_plan)
                .map_err(host_session_launch_denial)?;
        let host_plan_binding = host_session.plan_binding();
        let runtime =
            match Self::launch(
                WorthUiRuntimeLaunch {
                    artifact,
                    frame_epoch: crate::runtime::WorthUiRuntimeFrameEpoch::initial(),
                    diagnostic_policy,
                    candidate_snapshot_digest: Some(snapshot_digest.as_u64()),
                    candidate_artifact_digest: Some(artifact_digest),
                },
                WorthUiRuntimeLaunchAuthority {
                    lowering_authority,
                    initial_allocation_commit,
                    snapshot_digest,
                    retained_allocation_planning_evidence,
                    query_binding,
                    host_plan_binding,
                    change_profile,
                },
            ) {
                Ok(runtime) => runtime,
                Err(cause) => {
                    return match host_session.release_adapter_session() {
                    worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(receipt)
                        if receipt.released_surface_count() == 0 => Err(cause),
                    worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(receipt) => {
                        Err(WorthUiRuntimeLaunchDenial::HostSessionReleaseMismatch {
                            cause: Box::new(cause),
                            released_surface_count: receipt.released_surface_count(),
                        })
                    }
                    worth_ui_host_contract::UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                        _,
                    ) => Err(
                        WorthUiRuntimeLaunchDenial::HostSessionReleaseIndeterminate {
                            cause: Box::new(cause),
                            recovery: Box::new(
                                crate::facade::WorthUiHostSessionReleaseRecovery::retain(
                                    host_session,
                                ),
                            ),
                        },
                    ),
                };
                }
            };
        Ok((runtime, host_session))
    }

    pub(crate) fn launch(
        launch: WorthUiRuntimeLaunch,
        authority: WorthUiRuntimeLaunchAuthority,
    ) -> Result<Self, WorthUiRuntimeLaunchDenial> {
        let WorthUiRuntimeLaunch {
            artifact,
            frame_epoch,
            diagnostic_policy,
            candidate_snapshot_digest,
            candidate_artifact_digest,
        } = launch;
        let WorthUiRuntimeLaunchAuthority {
            lowering_authority,
            initial_allocation_commit,
            snapshot_digest,
            retained_allocation_planning_evidence,
            query_binding,
            host_plan_binding,
            change_profile,
        } = authority;
        let arena_identity = crate::runtime::WorthUiHandleArenaIdentity::from_host_session(
            host_plan_binding.session_identity(),
        );
        if let Some(candidate_snapshot_digest) = candidate_snapshot_digest
            && candidate_snapshot_digest != snapshot_digest.as_u64()
        {
            return Err(WorthUiRuntimeLaunchDenial::CandidateSnapshotMismatch {
                candidate_snapshot_digest,
                app_snapshot_digest: snapshot_digest.as_u64(),
            });
        }
        let (active_artifact, artifact_digest) = match candidate_artifact_digest {
            Some(artifact_digest) => (
                crate::runtime::active::WorthUiActiveArtifact::new(artifact, artifact_digest),
                artifact_digest,
            ),
            None => seal_launch_artifact(artifact),
        };
        let application_lowering_authority = lowering_authority;
        let plan_lowering_authority =
            crate::runtime::planning::WorthUiExecutionPlanLoweringAuthority::seal_launch(
                application_lowering_authority.clone(),
                initial_allocation_commit,
                active_artifact.artifact(),
                artifact_digest,
                frame_epoch,
            )
            .map_err(map_launch_lowering_denial)?;
        let handles =
            crate::runtime::execution::handle_allocation::WorthUiRuntimeHandleAllocator::allocate(
                plan_lowering_authority.facts(),
                arena_identity,
            )
            .map_err(WorthUiRuntimeLaunchDenial::HandleAllocation)?;
        let (candidate_plan, lane_admission) = crate::runtime::planning::plan_topology::WorthUiPlanTopologyAssembler::assemble_from_authority_with_lane_admission(
                plan_lowering_authority.facts(),
                &handles,
            )
            .map_err(WorthUiRuntimeLaunchDenial::TopologyAssembly)?;
        let plan_bundle = crate::runtime::active::WorthUiSealedExecutionPlanBundle::seal(
            plan_lowering_authority.facts(),
            candidate_plan,
            &lane_admission,
            host_plan_binding,
        )
        .map_err(map_plan_bundle_denial)?;
        let active_plan =
            crate::runtime::active::WorthUiActiveExecutionPlan::from_lowered_bundle(plan_bundle);
        plan_lowering_authority.finish_launch();
        let active = build_active_runtime_state(
            active_artifact,
            active_plan,
            snapshot_digest,
            frame_epoch,
            diagnostic_policy,
        );
        let last_valid = WorthUiLastValidRuntimeState::record_from_active(&active);

        let allocation_frame_scheduler = UiAllocationFrameFrameworkScheduler::launch(frame_epoch);
        Ok(Self {
            active_application_lowering_authority: application_lowering_authority,
            active,
            last_valid,
            retained_allocation_planning_evidence,
            allocation_receipt_ledger: UiAllocationReceiptLedger::for_runtime_generation(
                artifact_digest.raw(),
            ),
            allocation_invalidation_index: RefCell::new(Default::default()),
            allocation_frame_scheduler,
            allocation_source_order_ledger: Default::default(),
            query_binding,
            transient_interaction_admission: Default::default(),
            host_measurement_source: Rc::new(RefCell::new(Default::default())),
            host_session_identity: Some(host_plan_binding.session_identity()),
            host_observation_generation: Some(host_plan_binding.observation_generation()),
            host_plan_binding,
            durable_resize_source: Default::default(),
            observation: crate::runtime::observation::UiObservationRuntimeState::new(),
            service_proposals:
                crate::runtime::session::service_proposal::UiServiceProposalCompiler::new(),
            change_profile,
        })
    }
}

fn host_session_launch_denial(
    denial: crate::facade::WorthUiHostSessionActivationDenial,
) -> WorthUiRuntimeLaunchDenial {
    match denial {
        crate::facade::WorthUiHostSessionActivationDenial::IdentityExhausted => {
            WorthUiRuntimeLaunchDenial::HostSessionIdentityExhausted
        }
        crate::facade::WorthUiHostSessionActivationDenial::Protocol(denial) => {
            WorthUiRuntimeLaunchDenial::HostProtocol(denial)
        }
        crate::facade::WorthUiHostSessionActivationDenial::MountedPresentationLease(_) => {
            WorthUiRuntimeLaunchDenial::HostMountedPresentationLease
        }
        crate::facade::WorthUiHostSessionActivationDenial::ObservationSession(denial) => {
            WorthUiRuntimeLaunchDenial::HostObservationSession(denial)
        }
    }
}

fn map_plan_bundle_denial(
    denial: crate::runtime::active::WorthUiExecutionPlanBundleDenial,
) -> WorthUiRuntimeLaunchDenial {
    match denial {
        crate::runtime::active::WorthUiExecutionPlanBundleDenial::ForeignLoweringAuthority => {
            WorthUiRuntimeLaunchDenial::ExecutionPlanAuthorityMismatch
        }
        crate::runtime::active::WorthUiExecutionPlanBundleDenial::OrdinaryPlan(denial) => {
            WorthUiRuntimeLaunchDenial::OrdinaryPlan(denial)
        }
        crate::runtime::active::WorthUiExecutionPlanBundleDenial::VirtualizedPlan(denial) => {
            WorthUiRuntimeLaunchDenial::VirtualizedPlan(denial)
        }
        crate::runtime::active::WorthUiExecutionPlanBundleDenial::CanvasSpatialPlan(denial) => {
            WorthUiRuntimeLaunchDenial::CanvasSpatialPlan(denial)
        }
        crate::runtime::active::WorthUiExecutionPlanBundleDenial::RealtimeOverlayPlan(denial) => {
            WorthUiRuntimeLaunchDenial::RealtimeOverlayPlan(denial)
        }
    }
}

fn map_launch_lowering_denial(
    denial: crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial,
) -> WorthUiRuntimeLaunchDenial {
    match denial {
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::CandidateGraphAuthorityMismatch => {
            WorthUiRuntimeLaunchDenial::CandidateGraphAuthorityMismatch
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::CandidateArtifactAuthorityMismatch => {
            WorthUiRuntimeLaunchDenial::CandidateArtifactAuthorityMismatch
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::ForeignAllocationProjection => {
            WorthUiRuntimeLaunchDenial::ForeignAllocationProjection
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::MissingQueryPosture => {
            WorthUiRuntimeLaunchDenial::MissingQueryPosture
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::UnexpectedQueryPosture => {
            WorthUiRuntimeLaunchDenial::UnexpectedQueryPosture
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::QueryDefinitionNotInstalled => {
            WorthUiRuntimeLaunchDenial::QueryDefinitionNotInstalled
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::ForeignQueryInstalledAuthority => {
            WorthUiRuntimeLaunchDenial::ForeignQueryInstalledAuthority
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::RegionalDelta(denial) => {
            match denial {
                crate::runtime::planning::plan_topology::WorthUiPlanRegionDeltaDenial::DuplicateCandidateRegion => WorthUiRuntimeLaunchDenial::RegionalDeltaDuplicateCandidateRegion,
            }
        }
        crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::PlanInput(denial) => {
            WorthUiRuntimeLaunchDenial::PlanInput(denial)
        }
    }
}
