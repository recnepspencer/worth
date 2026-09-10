#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum BranchName {
    Root,
    Reuse,
    Relational,
    Signal,
    Independent,
}

impl BranchName {
    pub(super) const NON_ROOT: [Self; 4] = [
        Self::Reuse,
        Self::Relational,
        Self::Signal,
        Self::Independent,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Reuse => "reuse",
            Self::Relational => "relational",
            Self::Signal => "signal",
            Self::Independent => "independent",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BranchPosture {
    Reuse,
    Relational,
    Signal,
    Independent,
}

impl BranchPosture {
    pub(super) const ALL: [Self; 4] = [
        Self::Reuse,
        Self::Relational,
        Self::Signal,
        Self::Independent,
    ];

    pub(super) const fn branch(self) -> BranchName {
        match self {
            Self::Reuse => BranchName::Reuse,
            Self::Relational => BranchName::Relational,
            Self::Signal => BranchName::Signal,
            Self::Independent => BranchName::Independent,
        }
    }

    pub(super) const fn index(self) -> usize {
        match self {
            Self::Reuse => 0,
            Self::Relational => 1,
            Self::Signal => 2,
            Self::Independent => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum EffectPosture {
    Relational,
    Signal,
    Combined,
}

impl EffectPosture {
    const REQUIRED: [Self; 3] = [Self::Relational, Self::Signal, Self::Combined];

    pub(super) const fn installed_definition_count(self) -> usize {
        match self {
            Self::Relational => 0,
            Self::Signal => 2,
            Self::Combined => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Action {
    Fork(BranchPosture),
    Select(BranchPosture),
    Read(BranchName),
    Effect(EffectAction),
    Retire(BranchName),
    VerifyCleanup,
}

impl Action {
    pub(super) fn label(&self) -> String {
        match self {
            Self::Fork(posture) => format!("fork:{posture:?}"),
            Self::Select(posture) => format!("select:{posture:?}"),
            Self::Read(branch) => format!("read:{}", branch.label()),
            Self::Effect(EffectAction {
                target,
                observer,
                posture,
                input,
                ordinal,
            }) => format!(
                "effect:{posture:?}:target={}:observer={}:input={input}:ordinal={ordinal}",
                target.label(),
                observer.label()
            ),
            Self::Retire(branch) => format!("retire:{}", branch.label()),
            Self::VerifyCleanup => "verify-cleanup".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct EffectAction {
    pub(super) target: BranchName,
    pub(super) observer: BranchName,
    pub(super) posture: EffectPosture,
    pub(super) input: String,
    pub(super) ordinal: u8,
}

pub(super) fn sequence(seed: u64, rounds: usize) -> Vec<Action> {
    assert!(rounds >= EffectPosture::REQUIRED.len());
    let mut random = SeededOrder::new(seed);
    let mut actions = Vec::with_capacity(BranchPosture::ALL.len() * 4 + rounds + 1);

    let mut forks = BranchPosture::ALL;
    random.shuffle(&mut forks);
    actions.extend(forks.into_iter().map(Action::Fork));

    let mut selections = BranchPosture::ALL;
    random.shuffle(&mut selections);
    actions.extend(selections.into_iter().map(Action::Select));

    let mut reads = BranchName::NON_ROOT;
    random.shuffle(&mut reads);
    actions.extend(reads.into_iter().map(Action::Read));

    let mut effects = Vec::with_capacity(rounds);
    effects.extend(EffectPosture::REQUIRED);
    while effects.len() < rounds {
        effects.push(EffectPosture::REQUIRED[random.index(EffectPosture::REQUIRED.len())]);
    }
    random.shuffle(&mut effects);
    for (round, posture) in effects.into_iter().enumerate() {
        let target = match posture {
            EffectPosture::Relational => {
                [BranchName::Relational, BranchName::Independent][random.index(2)]
            }
            EffectPosture::Signal => [BranchName::Signal, BranchName::Independent][random.index(2)],
            EffectPosture::Combined => BranchName::Independent,
        };
        let target_index = BranchName::NON_ROOT
            .iter()
            .position(|branch| *branch == target)
            .expect("effect target belongs to the live non-root roster");
        let observer_offset = 1 + random.index(BranchName::NON_ROOT.len() - 1);
        let observer =
            BranchName::NON_ROOT[(target_index + observer_offset) % BranchName::NON_ROOT.len()];
        actions.push(Action::Effect(EffectAction {
            target,
            observer,
            posture,
            input: format!("seed-{seed:016x}-round-{round}"),
            ordinal: u8::try_from(round + 1).expect("bounded model ordinal must fit u8"),
        }));
    }

    let mut retirements = BranchName::NON_ROOT;
    random.shuffle(&mut retirements);
    actions.extend(retirements.into_iter().map(Action::Retire));
    actions.push(Action::VerifyCleanup);
    actions
}

struct SeededOrder {
    state: u64,
}

impl SeededOrder {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn index(&mut self, upper: usize) -> usize {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state as usize) % upper
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for upper in (1..values.len()).rev() {
            let index = self.index(upper + 1);
            values.swap(upper, index);
        }
    }
}
