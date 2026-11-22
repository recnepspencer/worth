//! A mixed conditional-definition original killed at each durable seam: open,
//! planned Signal leg, Native's effect, Signal's effect and the occurrence.
use super::model::Original;
use super::*;
use crate::runtime_world::court::retention::owner_evidence::mixed_definition;

#[derive(Clone, Copy, Debug)]
pub(super) enum Cut {
    /// Open, both owner effects and the occurrence are durable.
    Acknowledged,
    /// The process dies after the durable open, before the planned leg.
    OpenOnly,
    /// The planned leg lands, but its fate is unknown: Native never begins.
    LegOnly,
    /// Native's effect is durable; Signal's never becomes durable.
    NativeOnly,
    /// Both owner effects are durable; the occurrence is not.
    BothDurable,
}

pub(super) fn cut() -> impl Strategy<Value = Cut> {
    prop_oneof![
        Just(Cut::Acknowledged),
        Just(Cut::OpenOnly),
        Just(Cut::LegOnly),
        Just(Cut::NativeOnly),
        Just(Cut::BothDurable),
    ]
}

impl Model {
    /// Run one mixed original to its cut. Returns whether the process died.
    pub(super) fn attempt_mixed(
        &mut self,
        fixture: &Fixture,
        request: &ExecutionRequest<'_, '_>,
        cut: Cut,
    ) -> bool {
        let head = root(fixture);
        let original =
            mixed_definition::prepare(fixture, request, &head, &format!("mixed-{}", self.writes));
        let identities = original.identities;
        self.admit_identities(identities);
        match cut {
            Cut::Acknowledged => {}
            Cut::OpenOnly => fixture.recorder.crash(),
            Cut::LegOnly => fixture.recorder.undecide_next(true),
            Cut::NativeOnly => {
                fixture.hold_signal_advances();
                fixture.recorder.crash_after(1);
            }
            Cut::BothDurable => fixture.recorder.crash_after(1),
        }
        let outcome = mixed_definition::publish(
            &fixture.world.publication_port(),
            request,
            original,
            &RuntimeWorldCancellationSource::new().token(),
        );
        let custody = match (cut, outcome) {
            (Cut::Acknowledged, WorldDurablePublicationOutcome::Performed(performed)) => {
                drop(performed.consume());
                self.generation += 1;
                self.native_commits += 1;
                self.signal_commits += 1;
                return false;
            }
            (
                Cut::OpenOnly | Cut::LegOnly,
                WorldDurablePublicationOutcome::RecoveryRequired {
                    cause: WorldDurablePublicationDenial::PlannedSignalLeg(_),
                    custody,
                },
            ) => {
                assert!(custody
                    .component_results()
                    .relational_settlement()
                    .is_none());
                None
            }
            (
                Cut::NativeOnly | Cut::BothDurable,
                WorldDurablePublicationOutcome::RecoveryRequired { custody, .. }
                | WorldDurablePublicationOutcome::ProductUnpublished(custody),
            ) => Some(custody),
            (cut, other) => panic!("{cut:?} reached {other:?}"),
        };
        fixture.recorder.crash();
        let receipt = custody.map(|custody| {
            self.native_commits += 1;
            custody
                .component_results()
                .relational_settlement()
                .cloned()
                .expect("Native's effect precedes the lost Signal leg or occurrence")
        });
        let signal = matches!(cut, Cut::BothDurable);
        self.signal_commits += usize::from(signal);
        self.mixed.push(identities);
        self.originals.push(Original {
            identities,
            receipt,
            signal,
        });
        true
    }

    /// A Native-only mixed original still owes its planned Signal leg.
    pub(super) fn mixed_pending(&self, original: &Original) -> bool {
        self.mixed.contains(&original.identities) && original.receipt.is_some() && !original.signal
    }
}
