//! Phases 2, 3 & 4 of the C4b.4 Single-Home Unified 4-Family Oracle:
//! Governed Workflow (Instance #1 & Instance #2 -> `Historical`) + Core Host
//! follow-up writes + Branch Program Adoption (`P0 -> P1`) + D14 Discard +
//! Final Checkpointed Reopen on the SAME durable Home and SAME `MemoryOnly`
//! reference.
use super::barrier::kill_unified_barrier;
use super::child::UnifiedSession;
use super::ops::{
    advance_doc_wf_historical, exec_set_retention, exec_wf_correctable, run_unified_adopt_p1,
};
use super::second_wf::run_second_wf_to_historical;
use super::wire::{assert_unified_parity, UnifiedCmd};
use super::{RefHomeSlot, UnifiedHomeStepper};
use crate::process::application::host_oracle::adversarial::cumulative_oracle::StepCtx;
use crate::process::application::host_oracle::adversarial::dimensions_matrix::{
    assert_cadence_checkpointed, assert_cadence_discarded,
};
use crate::process::application::host_oracle::adversarial::workflow_correction_fixture::seed_approved_workflow;
use worth_execution::ExecutionRequest;
use worth_query_testkit_host::facade::host::checkpoint_capture;

const KEY_CORE_RESET: u64 = 1002;
const KEY_CORE_POST_WF2: u64 = 1003;

impl UnifiedHomeStepper {
    pub(super) fn step_workflow_and_d14(&mut self) {
        let mut s = self.session.take().unwrap();
        let ref_orig1 = {
            let (rw, rauth) = self.ref_slot.bind_wf();
            seed_approved_workflow(rw, rauth)
        };
        self.ref_wf_locators
            .push(ref_orig1.to_recorded_bytes().unwrap());
        let seeded = s.send(UnifiedCmd::SeedWorkflow);
        self.durable_wf_locators
            .push(seeded.wf_locator.expect("durable wf#1 locator"));
        let seed_ctx = self.ctx(2, "seed-workflow", "none");
        assert_unified_parity(
            &mut self.map,
            seeded.snap.as_ref().unwrap(),
            &self.ref_slot.observe(&self.ref_wf_locators),
            &seed_ctx,
        );
        let pre_ctx = self.ctx(2, "pre-wf-kill", "kill_live");
        s.kill_live(&pre_ctx.to_string());
        let nth = 1 + (self.rng.range(16) as u64);
        let kill_build = self.rng.next_other_build(self.build);
        let kill_ctx = StepCtx {
            build: kill_build,
            ..self.ctx(
                2,
                "wf-correctable-kill",
                format!("SynchronizeFileState#{nth}"),
            )
        };
        kill_unified_barrier(
            &self.home_dir,
            "wf",
            kill_build,
            nth,
            &self.coords_file,
            &kill_ctx.to_string(),
        );
        self.build = self.rng.next_other_build(kill_build);
        let ctx = self.ctx(2, "wf-reopen", format!("SynchronizeFileState#{nth}"));
        let ch = self.next_ch("post-wf");
        let (mut s2, o2) =
            UnifiedSession::start(&self.home_dir, &ch, self.build, &self.durable_wf_locators);
        let snap2 = o2.snap.unwrap();
        let ref_before = self.ref_slot.observe(&self.ref_wf_locators);
        assert!(
            snap2.wf_states.len() == 1 && snap2.wf_states[0].starts_with("Live:"),
            "{ctx}: wf#1 must be Live before repair (got {:?})",
            snap2.wf_states
        );
        assert_eq!(
            snap2.branches[1].retention, self.fork_days,
            "{ctx}: fork survived"
        );
        {
            let RefHomeSlot::Wf(w, _) = &self.ref_slot else {
                unreachable!();
            };
            assert_eq!(exec_wf_correctable(w), "committed", "{ctx}: MemoryOnly wf");
        }
        let ref_after = self.ref_slot.observe(&self.ref_wf_locators);
        if snap2.custody_count > 0 {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let u = s2.send(UnifiedCmd::RunCorrectableStep);
            assert_eq!(u.fate_tag.as_deref(), Some("product_unpublished"), "{ctx}");
            let cont = s2.send(UnifiedCmd::ContinueCustody);
            assert_unified_parity(&mut self.map, cont.snap.as_ref().unwrap(), &ref_after, &ctx);
            let r = s2.send(UnifiedCmd::RunCorrectableStep);
            assert!(
                matches!(
                    r.fate_tag.as_deref(),
                    Some("recovered_committed" | "already_committed")
                ),
                "{ctx}: post-cont wf retry: {:?}",
                r.fate_tag
            );
        } else if snap2.branches[0].retention == ref_before.branches[0].retention {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let r = s2.send(UnifiedCmd::RunCorrectableStep);
            assert_eq!(r.fate_tag.as_deref(), Some("committed"), "{ctx}");
            assert_unified_parity(&mut self.map, r.snap.as_ref().unwrap(), &ref_after, &ctx);
        } else {
            assert_unified_parity(&mut self.map, &snap2, &ref_after, &ctx);
            let r = s2.send(UnifiedCmd::RunCorrectableStep);
            assert!(
                matches!(
                    r.fate_tag.as_deref(),
                    Some("recovered_committed" | "already_committed")
                ),
                "{ctx}: committed wf retry"
            );
        }

        let d14_kill = self.ctx(2, "wf-d14-kill", "kill_live");
        let d14_r = s2.send(UnifiedCmd::CheckpointAndDiscard);
        assert!(d14_r.discard_gen.is_some(), "{d14_kill}: discard_gen");
        assert_unified_parity(
            &mut self.map,
            d14_r.snap.as_ref().unwrap(),
            &ref_after,
            &d14_kill,
        );
        self.discarded = true;
        s2.kill_live(&d14_kill.to_string());
        self.build = self.rng.next_other_build(self.build);
        let d14_ctx = self.ctx(2, "wf-d14-reopen", "kill_live");
        let ch_d = self.next_ch("wf-d14");
        let (s3, o3) =
            UnifiedSession::start(&self.home_dir, &ch_d, self.build, &self.durable_wf_locators);
        let snap3 = o3.snap.unwrap();
        assert_cadence_discarded(&snap3.cadence, &d14_ctx.to_string());
        assert_unified_parity(&mut self.map, &snap3, &ref_after, &d14_ctx);
        self.session = Some(s3);
        self.phase = 3;
    }

    pub(super) fn step_second_workflow_and_historical(&mut self) {
        let mut s = self.session.take().unwrap();
        let ctx = self.ctx(3, "wf1-historical+wf2-historical+d14", "kill_live");
        {
            let RefHomeSlot::Wf(w, _) = &self.ref_slot else {
                unreachable!();
            };
            advance_doc_wf_historical(w);
        }
        let hist = s.send(UnifiedCmd::AdvanceWorkflowHistorical);
        assert_unified_parity(
            &mut self.map,
            hist.snap.as_ref().unwrap(),
            &self.ref_slot.observe(&self.ref_wf_locators),
            &ctx,
        );
        // Reset main retention back to `main_days`, run Workflow Instance #2 across
        // assessments/approval/CorrectableRetentionBinding -> Historical on the SAME
        // post-D14 Home, restore `main_days` via Core Host SetRetention, then D14
        // discard + live OS kill + reopen on alternating build verifying BOTH
        // historical workflow instances remain readable and equal to MemoryOnly!
        let _ = exec_set_retention(self.ref_slot.runtime(), 0, self.main_days, KEY_CORE_RESET);
        let reset = s.send(UnifiedCmd::SetRetention {
            branch_idx: 0,
            days: self.main_days,
            key: KEY_CORE_RESET,
        });
        assert_eq!(reset.fate_tag.as_deref(), Some("committed"), "{ctx}: reset");
        let ref_orig2 = {
            let RefHomeSlot::Wf(w, a) = &self.ref_slot else {
                unreachable!();
            };
            run_second_wf_to_historical(w, a)
        };
        self.ref_wf_locators
            .push(ref_orig2.to_recorded_bytes().unwrap());
        let wf2_reply = s.send(UnifiedCmd::RunSecondWfToHistorical);
        self.durable_wf_locators
            .push(wf2_reply.wf_locator.expect("durable wf#2 locator"));
        assert_unified_parity(
            &mut self.map,
            wf2_reply.snap.as_ref().unwrap(),
            &self.ref_slot.observe(&self.ref_wf_locators),
            &ctx,
        );
        let _ = exec_set_retention(
            self.ref_slot.runtime(),
            0,
            self.main_days,
            KEY_CORE_POST_WF2,
        );
        let restored = s.send(UnifiedCmd::SetRetention {
            branch_idx: 0,
            days: self.main_days,
            key: KEY_CORE_POST_WF2,
        });
        assert_eq!(
            restored.fate_tag.as_deref(),
            Some("committed"),
            "{ctx}: post-wf2"
        );
        let ref_after = self.ref_slot.observe(&self.ref_wf_locators);
        assert!(
            ref_after.wf_states.len() == 2
                && ref_after
                    .wf_states
                    .iter()
                    .all(|st| st.starts_with("Historical:")),
            "{ctx}: both workflow instances must be Historical (got {:?})",
            ref_after.wf_states
        );
        assert_eq!(
            ref_after.branches[0].retention, self.main_days,
            "{ctx}: restored"
        );
        assert_unified_parity(
            &mut self.map,
            restored.snap.as_ref().unwrap(),
            &ref_after,
            &ctx,
        );

        let d14_kill = self.ctx(3, "wf2-d14-kill", "kill_live");
        let d14_r = s.send(UnifiedCmd::CheckpointAndDiscard);
        assert!(d14_r.discard_gen.is_some(), "{d14_kill}: discard_gen");
        assert_unified_parity(
            &mut self.map,
            d14_r.snap.as_ref().unwrap(),
            &ref_after,
            &d14_kill,
        );
        self.discarded = true;
        s.kill_live(&d14_kill.to_string());
        self.build = self.rng.next_other_build(self.build);
        let d14_ctx = self.ctx(3, "wf2-d14-reopen", "kill_live");
        let ch_d = self.next_ch("wf2-d14");
        let (s2, o2) =
            UnifiedSession::start(&self.home_dir, &ch_d, self.build, &self.durable_wf_locators);
        let snap2 = o2.snap.unwrap();
        assert_cadence_discarded(&snap2.cadence, &d14_ctx.to_string());
        assert_eq!(
            snap2.branches[0].retention, self.main_days,
            "{d14_ctx}: restored survived"
        );
        assert_eq!(
            snap2.branches[1].retention, self.fork_days,
            "{d14_ctx}: fork survived"
        );
        assert_unified_parity(&mut self.map, &snap2, &ref_after, &d14_ctx);
        self.session = Some(s2);
        self.phase = 4;
    }

    pub(super) fn step_adoption_and_final(&mut self) {
        let lease = checkpoint_capture::lease(256 << 20);
        let exec = ExecutionRequest::leased(&lease);
        let mut s = self.session.take().unwrap();
        let pre_ctx = self.ctx(4, "pre-adopt-kill", "kill_live");
        s.kill_live(&pre_ctx.to_string());
        let nth = 1 + (self.rng.range(16) as u64);
        let kill_build = self.rng.next_other_build(self.build);
        let kill_ctx = StepCtx {
            build: kill_build,
            ..self.ctx(4, "adopt-p1-kill", format!("SynchronizeFileState#{nth}"))
        };
        kill_unified_barrier(
            &self.home_dir,
            "adopt",
            kill_build,
            nth,
            &self.coords_file,
            &kill_ctx.to_string(),
        );
        self.build = self.rng.next_other_build(kill_build);
        let ctx = self.ctx(4, "adopt-reopen", format!("SynchronizeFileState#{nth}"));
        let ch = self.next_ch("post-adopt");
        let (mut s2, o2) =
            UnifiedSession::start(&self.home_dir, &ch, self.build, &self.durable_wf_locators);
        let snap2 = o2.snap.unwrap();
        let ref_before = self.ref_slot.observe(&self.ref_wf_locators);
        assert_eq!(
            snap2.branches[0].retention, self.main_days,
            "{ctx}: main survived"
        );
        assert_eq!(
            snap2.branches[1].retention, self.fork_days,
            "{ctx}: fork survived"
        );
        run_unified_adopt_p1(self.ref_slot.runtime());
        let ref_after = self.ref_slot.observe(&self.ref_wf_locators);
        assert!(
            ref_after.wf_states.len() == 2
                && ref_after
                    .wf_states
                    .iter()
                    .all(|st| st.starts_with("Historical:")),
            "{ctx}: both historical workflow instances must remain readable after P1 adoption (got {:?})",
            ref_after.wf_states
        );
        assert_ne!(
            ref_after.branches[0].program_rev, ref_after.branches[1].program_rev,
            "{ctx}: main on P1 while fork stays on P0"
        );
        if snap2.custody_count > 0 {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let cont = s2.send(UnifiedCmd::ContinueCustody);
            assert_unified_parity(&mut self.map, cont.snap.as_ref().unwrap(), &ref_after, &ctx);
        } else if snap2.branches[0].program_rev == ref_before.branches[0].program_rev {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let ad = s2.send(UnifiedCmd::AdoptP1);
            assert_unified_parity(&mut self.map, ad.snap.as_ref().unwrap(), &ref_after, &ctx);
        } else {
            assert_unified_parity(&mut self.map, &snap2, &ref_after, &ctx);
        }

        let d14_kill = self.ctx(4, "adopt-d14-kill", "kill_live");
        let d14_r = s2.send(UnifiedCmd::CheckpointAndDiscard);
        assert!(d14_r.discard_gen.is_some(), "{d14_kill}: discard_gen");
        assert_unified_parity(
            &mut self.map,
            d14_r.snap.as_ref().unwrap(),
            &ref_after,
            &d14_kill,
        );
        self.discarded = true;
        s2.kill_live(&d14_kill.to_string());
        self.build = self.rng.next_other_build(self.build);
        let d14_ctx = self.ctx(4, "adopt-d14-reopen", "kill_live");
        let ch_d = self.next_ch("adopt-d14");
        let (mut s3, o3) =
            UnifiedSession::start(&self.home_dir, &ch_d, self.build, &self.durable_wf_locators);
        let snap3 = o3.snap.unwrap();
        assert_cadence_discarded(&snap3.cadence, &d14_ctx.to_string());
        assert_unified_parity(&mut self.map, &snap3, &ref_after, &d14_ctx);
        let closed = s3.send(UnifiedCmd::Close {
            expect_discarded: false,
        });
        assert!(closed.closed && closed.close_gen.is_some(), "{d14_ctx}");
        s3.finish();

        self.build = self.rng.next_other_build(self.build);
        let fin_ctx = self.ctx(4, "final-checkpoint-reopen", "none");
        let ch_fin = self.next_ch("final");
        let (mut fin, ofin) = UnifiedSession::start(
            &self.home_dir,
            &ch_fin,
            self.build,
            &self.durable_wf_locators,
        );
        let fin_snap = ofin.snap.unwrap();
        assert_cadence_checkpointed(&fin_snap.cadence, &fin_ctx.to_string());
        assert_unified_parity(&mut self.map, &fin_snap, &ref_after, &fin_ctx);
        assert!(
            fin.send(UnifiedCmd::Close {
                expect_discarded: false
            })
            .closed
        );
        fin.finish();
        match std::mem::replace(&mut self.ref_slot, RefHomeSlot::Closed) {
            RefHomeSlot::Raw(h) => drop(h.close(&exec).err().unwrap().resume().unwrap()),
            RefHomeSlot::Wf(w, a) => {
                drop(a);
                drop(w.close(&exec).err().unwrap().resume().unwrap());
            }
            RefHomeSlot::Closed => {}
        }
        self.phase = 5;
    }
}
