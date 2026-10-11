use worth_query::facade::{
    foundation::WorthQueryEntityIdentity,
    installed::collection::WorthQueryCollectionDeliveryDenialKind, runtime::WorthQueryWorkspace,
};

use crate::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionFactReceipt, UiCollectionProjectionOpenOutcome,
    UiCollectionProjectionRegistration, UiLiveCollectionProjection,
    UiLiveCollectionProjectionCloseOutcome, UiProjectionFieldRequirement, WorthUiQueryWorkspaceExt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiCollectionPatchAttack {
    Duplicate,
    Reordered,
    Superseded,
    ForeignLease,
    WrongWindow,
}

pub struct WorthUiCollectionPatchAttackReport {
    attack: WorthUiCollectionPatchAttack,
    denial: WorthQueryCollectionDeliveryDenialKind,
    state_preserved: bool,
    follow_up_delivery_succeeded: bool,
    successful_facts: Box<[UiCollectionProjectionFactReceipt]>,
    closed_resources: usize,
    terminal_owners: usize,
}

impl WorthUiCollectionPatchAttackReport {
    pub fn attack(&self) -> WorthUiCollectionPatchAttack {
        self.attack
    }

    pub fn denial(&self) -> WorthQueryCollectionDeliveryDenialKind {
        self.denial
    }

    pub fn state_preserved(&self) -> bool {
        self.state_preserved
    }

    pub fn follow_up_delivery_succeeded(&self) -> bool {
        self.follow_up_delivery_succeeded
    }

    pub fn successful_facts(&self) -> &[UiCollectionProjectionFactReceipt] {
        &self.successful_facts
    }

    pub fn closed_resources(&self) -> usize {
        self.closed_resources
    }

    pub fn terminal_owners(&self) -> usize {
        self.terminal_owners
    }
}

pub fn certify_collection_patch_attack(
    attack: WorthUiCollectionPatchAttack,
) -> WorthUiCollectionPatchAttackReport {
    let mut world = PatchAttackWorld::new(attack);
    let outcome = match attack {
        WorthUiCollectionPatchAttack::Duplicate => world.duplicate(),
        WorthUiCollectionPatchAttack::Reordered => world.reordered(),
        WorthUiCollectionPatchAttack::Superseded => world.superseded(),
        WorthUiCollectionPatchAttack::ForeignLease => world.foreign_lease(),
        WorthUiCollectionPatchAttack::WrongWindow => world.wrong_window(),
    };
    world.finish(attack, outcome)
}

struct PatchAttackOutcome {
    denial: WorthQueryCollectionDeliveryDenialKind,
    state_preserved: bool,
    follow_up_delivery_succeeded: bool,
    successful_facts: Vec<UiCollectionProjectionFactReceipt>,
}

struct PatchAttackWorld {
    workspace: WorthQueryWorkspace,
    changed: WorthQueryEntityIdentity,
    subject: Option<UiLiveCollectionProjection>,
    candidate: Option<UiLiveCollectionProjection>,
}

impl PatchAttackWorld {
    fn new(attack: WorthUiCollectionPatchAttack) -> Self {
        let mut workspace = super::collection_projection_workspace();
        let changed =
            super::insert_projection_status(&mut workspace, "pulse.alpha", "Alpha before");
        super::insert_projection_status(&mut workspace, "pulse.bravo", "Bravo");
        let subject_rows = if attack == WorthUiCollectionPatchAttack::WrongWindow {
            2
        } else {
            8
        };
        let subject = Some(open_live(&mut workspace, subject_rows));
        let candidate = matches!(
            attack,
            WorthUiCollectionPatchAttack::ForeignLease | WorthUiCollectionPatchAttack::WrongWindow
        )
        .then(|| open_live(&mut workspace, 1));
        Self {
            workspace,
            changed,
            subject,
            candidate,
        }
    }

    fn duplicate(&mut self) -> PatchAttackOutcome {
        self.update("Alpha duplicate");
        let (accepted, duplicate) = self.subject_patch_twins();
        let receipt = self
            .subject_mut()
            .certification_apply_patch(accepted)
            .expect("first duplicate twin applies");
        let fact = self.subject().certification_derive_fact(&receipt);
        self.denied_outcome(duplicate, vec![fact], false, true)
    }

    fn reordered(&mut self) -> PatchAttackOutcome {
        self.update("Alpha order one");
        let (first, reordered) = self.subject_patch_twins();
        let first = self
            .subject_mut()
            .certification_apply_patch(first)
            .expect("first ordered patch applies");
        let first_fact = self.subject().certification_derive_fact(&first);
        self.update("Alpha order two");
        let (second, unused_twin) = self.subject_patch_twins();
        drop(unused_twin);
        let second = self
            .subject_mut()
            .certification_apply_patch(second)
            .expect("second ordered patch applies");
        let second_fact = self.subject().certification_derive_fact(&second);
        self.denied_outcome(reordered, vec![first_fact, second_fact], false, true)
    }

    fn superseded(&mut self) -> PatchAttackOutcome {
        self.update("Alpha superseded one");
        let (superseded, unused_twin) = self.subject_patch_twins();
        drop(unused_twin);
        self.update("Alpha superseded two");
        let (reset, unused_reset) = self.subject_patch_twins();
        drop(unused_reset);
        let mut outcome = self.denied_outcome(superseded, Vec::new(), false, false);
        let receipt = self
            .subject_mut()
            .certification_apply_patch(reset)
            .expect("replacement reset applies after superseded denial");
        outcome
            .successful_facts
            .push(self.subject().certification_derive_fact(&receipt));
        outcome.follow_up_delivery_succeeded = true;
        outcome
    }

    fn foreign_lease(&mut self) -> PatchAttackOutcome {
        self.update("Alpha foreign");
        let (subject_attack, subject_success) = self.subject_patch_twins();
        let (candidate_success, unused_candidate) = self.candidate_patch_twins();
        drop(unused_candidate);
        let candidate_receipt = self
            .candidate_mut()
            .certification_apply_patch(candidate_success)
            .expect("candidate lease patch applies");
        let candidate_fact = self
            .candidate()
            .certification_derive_fact(&candidate_receipt);
        let mut outcome = self.denied_outcome(subject_attack, vec![candidate_fact], true, true);
        let subject_receipt = self
            .subject_mut()
            .certification_apply_patch(subject_success)
            .expect("subject lease patch applies");
        outcome
            .successful_facts
            .push(self.subject().certification_derive_fact(&subject_receipt));
        outcome
    }

    fn wrong_window(&mut self) -> PatchAttackOutcome {
        self.update("Alpha wrong window");
        let (subject_success, wrong_window, target_success) = {
            let Self {
                workspace,
                subject,
                candidate,
                ..
            } = self;
            subject
                .as_mut()
                .expect("subject live")
                .certification_plan_patch_for_target(
                    candidate.as_mut().expect("candidate live"),
                    workspace,
                )
        };
        let receipt = self
            .subject_mut()
            .certification_apply_patch(subject_success)
            .expect("wide-window patch applies to its owner");
        let fact = self.subject().certification_derive_fact(&receipt);
        let mut outcome = self.denied_outcome(wrong_window, vec![fact], true, false);
        let receipt = self
            .candidate_mut()
            .certification_apply_patch(target_success)
            .expect("wrong-window denial must preserve the target's valid pending patch");
        let follow_up = self.candidate().certification_derive_fact(&receipt);
        super::collection_patch_recovery::assert_exact_update(
            &follow_up,
            &self.changed,
            "Alpha wrong window",
        );
        outcome.follow_up_delivery_succeeded = true;
        outcome
    }

    fn denied_outcome(
        &mut self,
        patch: worth_query::facade::installed::collection::WorthQueryCollectionPatch,
        successful_facts: Vec<UiCollectionProjectionFactReceipt>,
        candidate_target: bool,
        prove_follow_up: bool,
    ) -> PatchAttackOutcome {
        let state_before_denial = self.target(candidate_target).certification_state_snapshot();
        let denial = match self
            .target_mut(candidate_target)
            .certification_apply_patch(patch)
        {
            Err(denial) => denial,
            Ok(_) => panic!("hostile patch must be denied"),
        };
        let state_after_denial = self.target(candidate_target).certification_state_snapshot();
        let follow_up_delivery_succeeded =
            !prove_follow_up || self.prove_follow_up_delivery(candidate_target);
        PatchAttackOutcome {
            denial: denial.kind(),
            state_preserved: state_before_denial == state_after_denial,
            follow_up_delivery_succeeded,
            successful_facts,
        }
    }

    fn finish(
        mut self,
        attack: WorthUiCollectionPatchAttack,
        outcome: PatchAttackOutcome,
    ) -> WorthUiCollectionPatchAttackReport {
        let mut closed_resources = 0;
        let mut terminal_owners = 0;
        for live in [self.candidate.take(), self.subject.take()]
            .into_iter()
            .flatten()
        {
            let UiLiveCollectionProjectionCloseOutcome::Closed(closed) =
                live.close(&mut self.workspace)
            else {
                panic!("collection attack lease must close");
            };
            closed_resources += 1;
            terminal_owners += usize::from(closed.owner_terminal());
        }
        WorthUiCollectionPatchAttackReport {
            attack,
            denial: outcome.denial,
            state_preserved: outcome.state_preserved,
            follow_up_delivery_succeeded: outcome.follow_up_delivery_succeeded,
            successful_facts: outcome.successful_facts.into_boxed_slice(),
            closed_resources,
            terminal_owners,
        }
    }

    fn update(&mut self, value: &str) {
        super::update_projection_status(&mut self.workspace, self.changed.clone(), value);
    }

    fn prove_follow_up_delivery(&mut self, candidate_target: bool) -> bool {
        const FOLLOW_UP_VALUE: &str = "Alpha after denial";
        self.update(FOLLOW_UP_VALUE);
        let (patch, unused_twin) = if candidate_target {
            self.candidate_patch_twins()
        } else {
            self.subject_patch_twins()
        };
        drop(unused_twin);
        let receipt = self
            .target_mut(candidate_target)
            .certification_apply_patch(patch)
            .expect("a denied hostile patch must not poison the next valid delivery");
        let fact = self
            .target(candidate_target)
            .certification_derive_fact(&receipt);
        super::collection_patch_recovery::assert_exact_update(
            &fact,
            &self.changed,
            FOLLOW_UP_VALUE,
        );
        true
    }

    fn subject(&self) -> &UiLiveCollectionProjection {
        self.subject.as_ref().expect("subject live")
    }

    fn subject_mut(&mut self) -> &mut UiLiveCollectionProjection {
        self.subject.as_mut().expect("subject live")
    }

    fn candidate(&self) -> &UiLiveCollectionProjection {
        self.candidate.as_ref().expect("candidate live")
    }

    fn candidate_mut(&mut self) -> &mut UiLiveCollectionProjection {
        self.candidate.as_mut().expect("candidate live")
    }

    fn subject_patch_twins(
        &mut self,
    ) -> (
        worth_query::facade::installed::collection::WorthQueryCollectionPatch,
        worth_query::facade::installed::collection::WorthQueryCollectionPatch,
    ) {
        let Self {
            workspace, subject, ..
        } = self;
        subject
            .as_mut()
            .expect("subject live")
            .certification_plan_patch_twins(workspace)
    }

    fn candidate_patch_twins(
        &mut self,
    ) -> (
        worth_query::facade::installed::collection::WorthQueryCollectionPatch,
        worth_query::facade::installed::collection::WorthQueryCollectionPatch,
    ) {
        let Self {
            workspace,
            candidate,
            ..
        } = self;
        candidate
            .as_mut()
            .expect("candidate live")
            .certification_plan_patch_twins(workspace)
    }

    fn target(&self, candidate: bool) -> &UiLiveCollectionProjection {
        if candidate {
            self.candidate()
        } else {
            self.subject()
        }
    }

    fn target_mut(&mut self, candidate: bool) -> &mut UiLiveCollectionProjection {
        if candidate {
            self.candidate_mut()
        } else {
            self.subject_mut()
        }
    }
}

fn open_live(workspace: &mut WorthQueryWorkspace, max_rows: u32) -> UiLiveCollectionProjection {
    let installed = workspace.worth_ui().expect("WORTH UI domain installed");
    let registration = UiCollectionProjectionRegistration::text(
        installed
            .projection_view("certification.collection.patch")
            .expect("collection projection view"),
        UiProjectionFieldRequirement::declared("identity.id").expect("row identity field"),
        [UiProjectionFieldRequirement::declared("status").expect("selected field")],
        false,
        true,
    )
    .expect("collection projection registration");
    let UiCollectionProjectionBindingAdmission::Ready(binding) = registration.admit(workspace)
    else {
        panic!("collection projection binding must admit");
    };
    let budget = UiCollectionProjectionBudget::new(max_rows, 128, 1, 1_048_576)
        .expect("collection patch budget");
    let UiCollectionProjectionOpenOutcome::Opened(opened) = binding.open(budget, workspace) else {
        panic!("collection projection must open");
    };
    opened.into_parts().0
}
