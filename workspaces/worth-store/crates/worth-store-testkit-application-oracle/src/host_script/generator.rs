use super::{BranchLabel as Branch, CheckpointKind as Checkpoint, HostCommand as Command};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostScript {
    seed: u64,
    commands: Vec<Command>,
}
impl HostScript {
    /// Wrapping arithmetic is intentional: every u64 seed has a full script.
    pub fn generate(seed: u64) -> Self {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };
        let key = next();
        let value = next() % 100_000 + 10;
        let mut commands = vec![
            Command::Open,
            Command::Initialize,
            Command::Mutate {
                branch: Branch::Main,
                key: Some(key),
                value,
            },
            Command::Retry { key, value },
            Command::Retry {
                key,
                value: value + 1,
            },
            Command::Mutate {
                branch: Branch::Main,
                key: None,
                value: next() % 100_000,
            },
            Command::LoseResponse {
                key: key.wrapping_add(1),
                value: value + 2,
            },
            Command::Reopen,
            Command::Retry {
                key: key.wrapping_add(1),
                value: value + 2,
            },
            Command::CreateBranch(Branch::Left),
            Command::CreateBranch(Branch::Right),
        ];
        for branch in [Branch::Main, Branch::Left, Branch::Right] {
            commands.push(Command::MoveProduct {
                branch,
                value: next() % 100_000,
            });
            commands.push(Command::Mutate {
                branch,
                key: Some(next()),
                value: next() % 100_000,
            });
        }
        commands.extend([
            Command::SettlementFailure {
                key: key.wrapping_add(2),
                value: value + 3,
            },
            Command::Reopen,
            Command::ContinueCustody,
            Command::SettlementFailure {
                key: key.wrapping_add(3),
                value: value + 4,
            },
            Command::Reopen,
            Command::ReleaseCustody,
            Command::AbandonCaller {
                key: key.wrapping_add(4),
                value: value + 5,
            },
            Command::Reopen,
            Command::Checkpoint(Checkpoint::Explicit),
            Command::Checkpoint(Checkpoint::Automatic),
            Command::Checkpoint(Checkpoint::ImageAdmission),
            Command::ProgramTransition { revision: 2 },
            Command::ResumeTransition,
            Command::Close,
            Command::Reopen,
            Command::Close,
        ]);
        Self { seed, commands }
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}
