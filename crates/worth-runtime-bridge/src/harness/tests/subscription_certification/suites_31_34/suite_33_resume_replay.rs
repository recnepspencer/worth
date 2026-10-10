use super::super::support::*;
use crate::facade::{
    BridgeRuntimePolicy, BridgeSubscriptionDeliveryDensityPosture,
    BridgeSubscriptionDeliveryFamilyKind, BridgeSubscriptionDuplicateReplayPolicyKind,
    BridgeSubscriptionResumeAdmissionRejectionKind, RuntimeBridge,
};

#[test]
fn bridge_harness_subscription_suite_33_checkpoint_resume_and_replay_are_exact() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    for build_declaration in [
        detail_subscription as fn(&RuntimeBridge) -> crate::facade::BridgeSubscriptionDeclaration,
        collection_subscription,
    ] {
        let bridge = runtime(BridgeRuntimePolicy::development());
        let declaration = build_declaration(&bridge);
        let active = active_subscription_for(
            &bridge,
            &declaration,
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            1,
            resource_request,
        );
        let first_window = sealed_window_with_members(
            &bridge,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(2),
        );
        let second_window = sealed_window_with_members(
            &bridge,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            1,
            fixture_members(2),
        );
        let checkpoint_window = sealed_window_with_members(
            &bridge,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            2,
            fixture_members(2),
        );
        assert_ne!(first_window.digest(), second_window.digest());
        assert_ne!(second_window.digest(), checkpoint_window.digest());
        let checkpoint = checkpoint_from_sealed(
            &bridge,
            &active,
            &checkpoint_window,
            1,
            BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
        );
        let admission = bridge
            .admit_subscription_resume(&active, &checkpoint)
            .expect("resume admission should accept matching subscription checkpoint");
        let resume_plan = bridge.plan_subscription_resume(admission.clone());
        let retained =
            bridge.retain_subscription_delivery_window_seed(&sealed_window_with_members(
                &bridge,
                &active,
                BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
                3,
                fixture_members(1),
            ));
        let replay_plan = bridge
            .plan_subscription_delivery_replay(&active, admission, vec![retained])
            .expect("retained delivery replay should plan from subscription checkpoint");

        assert_eq!(
            resume_plan.active_subscription_identity(),
            active.active_subscription_identity()
        );
        assert_eq!(
            resume_plan.checkpoint_identity(),
            checkpoint.checkpoint_identity()
        );
        assert_eq!(resume_plan.expected_next_canonical_sequence(), 2);
        assert_eq!(
            replay_plan.active_subscription_identity(),
            active.active_subscription_identity()
        );
        assert_eq!(
            replay_plan.delivery_family_identity(),
            checkpoint.delivery_family_identity()
        );
        assert_eq!(replay_plan.retained_window_count(), 1);
        assert_eq!(replay_plan.retained_member_count(), 1);
        assert_eq!(replay_plan.counters().replay_mismatch_count(), 0);

        let restart_bridge = runtime(BridgeRuntimePolicy::development());
        let restart_declaration = build_declaration(&restart_bridge);
        let restart_active = active_subscription_for(
            &restart_bridge,
            &restart_declaration,
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            1,
            resource_request,
        );
        let restart_checkpoint_window = sealed_window_with_members(
            &restart_bridge,
            &restart_active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            2,
            fixture_members(2),
        );
        let restart_checkpoint = checkpoint_from_sealed(
            &restart_bridge,
            &restart_active,
            &restart_checkpoint_window,
            1,
            BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
        );
        let restart_admission = restart_bridge
            .admit_subscription_resume(&restart_active, &restart_checkpoint)
            .expect("restart resume admission should accept matching checkpoint");
        let restart_retained =
            restart_bridge.retain_subscription_delivery_window_seed(&sealed_window_with_members(
                &restart_bridge,
                &restart_active,
                BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
                3,
                fixture_members(1),
            ));
        let restart_replay_plan = restart_bridge
            .plan_subscription_delivery_replay(
                &restart_active,
                restart_admission,
                vec![restart_retained],
            )
            .expect("restart retained replay should plan from canonical artifacts");
        assert_eq!(checkpoint.digest(), restart_checkpoint.digest());
        assert_eq!(replay_plan.digest(), restart_replay_plan.digest());

        let stale_admission = bridge
            .admit_subscription_resume(&active, &checkpoint)
            .expect("resume admission should accept matching checkpoint for stale test");
        let stale_seed = bridge.retain_subscription_delivery_window_seed(&checkpoint_window);
        let stale_rejection = bridge
            .plan_subscription_delivery_replay(&active, stale_admission, vec![stale_seed])
            .expect_err("checkpoint window cannot be replayed as future retained work");
        assert_eq!(
            stale_rejection.rejection_kind(),
            crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::RetainedWindowNotAfterCheckpoint
        );

        let (_other_runtime, other_active) = {
            let other_runtime = runtime(BridgeRuntimePolicy::development());
            let other_declaration = detail_subscription(&other_runtime);
            let other_active = active_subscription_for(
                &other_runtime,
                &other_declaration,
                BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
                1,
                resource_request,
            );
            (other_runtime, other_active)
        };
        let resume_rejection = bridge
            .admit_subscription_resume(&other_active, &checkpoint)
            .expect_err("checkpoint from another active subscription must reject");
        assert_eq!(
            resume_rejection.rejection_kind(),
            BridgeSubscriptionResumeAdmissionRejectionKind::ActiveSubscriptionMismatch
        );
    }
}
