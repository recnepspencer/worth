use worth_ui_host_contract::{
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiSemanticTextSlot,
};

use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentRankedSequence};

#[derive(Clone, Default)]
pub(super) struct UiMountedPresentationCommandBundle {
    commands: UiPersistentOrdMap<UiMountedPresentationCommandKey, CommandRecord>,
    order: UiPersistentRankedSequence<UiMountedPresentationCommandKey>,
    identities:
        UiPersistentOrdMap<UiMountedPresentationIdentityKey, UiMountedPresentationCommandKey>,
    positions: UiPersistentOrdMap<UiMountedPresentationIdentityKey, usize>,
}

/// The optional raw sample occupies a slot in the admitted command record.
/// Candidate bundles remain private until the host accepts their work.
#[derive(Clone)]
struct CommandRecord {
    command: UiMountedPaintCommand,
    motion: super::motion_evidence::UiCommandMotionAcceptance,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UiMountedPresentationCommandKey {
    family: u8,
    slot: u16,
    collection: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UiMountedPresentationIdentityKey(u64, u64);

impl UiMountedPresentationCommandBundle {
    pub(super) fn from_commands(commands: &[UiMountedPaintCommand]) -> Self {
        let mut bundle = Self::default();
        for (position, command) in commands.iter().enumerate() {
            let key = UiMountedPresentationCommandKey::for_command(command);
            let identity = UiMountedPresentationIdentityKey::for_identity(command.identity());
            assert!(
                bundle.commands.get(&key).is_none(),
                "command keys are unique"
            );
            assert!(
                bundle.identities.get(&identity).is_none(),
                "command identity registry keys are unique"
            );
            bundle
                .order
                .insert(bundle.order.len(), key)
                .expect("bounded command bundle rank");
            bundle.commands.insert(
                key,
                CommandRecord {
                    command: command.clone(),
                    motion: Default::default(),
                },
            );
            bundle.identities.insert(identity, key);
            bundle.positions.insert(identity, position);
        }
        bundle
    }

    pub(super) fn iter(&self) -> impl ExactSizeIterator<Item = &UiMountedPaintCommand> {
        self.order.iter().map(|key| {
            &self
                .commands
                .get(key)
                .expect("command order names an indexed command")
                .command
        })
    }

    pub(super) fn appearance_motion(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &UiMountedPaintCommand,
            Option<super::super::motion_sampling::UiPresentationMotionSampleReceipt>,
        ),
    > {
        self.order.iter().map(|key| {
            let record = self
                .commands
                .get(key)
                .expect("command order names an indexed command");
            (&record.command, record.motion.sample())
        })
    }

    pub(super) fn get(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<&UiMountedPaintCommand> {
        let lookup = UiMountedPresentationIdentityKey::for_identity(identity);
        let key = self.identities.get(&lookup)?;
        self.commands
            .get(key)
            .map(|record| &record.command)
            .filter(|command| command.identity() == identity)
    }

    pub(super) fn position(&self, identity: UiMountedPaintCommandIdentity) -> Option<usize> {
        self.positions
            .get(&UiMountedPresentationIdentityKey::for_identity(identity))
            .copied()
    }

    pub(super) fn replace(&mut self, replacement: UiMountedPaintCommand) -> bool {
        let key = UiMountedPresentationCommandKey::for_command(&replacement);
        let identity = UiMountedPresentationIdentityKey::for_identity(replacement.identity());
        if self.identities.get(&identity) != Some(&key) {
            return false;
        }
        let Some(predecessor) = self.commands.get(&key) else {
            return false;
        };
        if predecessor.command.identity() != replacement.identity()
            || predecessor.command.layer_semantic_order() != replacement.layer_semantic_order()
        {
            return false;
        }
        let motion = super::command_same_presentation_meaning(&predecessor.command, &replacement)
            .then(|| predecessor.motion.clone())
            .unwrap_or_default();
        self.commands.insert(
            key,
            CommandRecord {
                command: replacement,
                motion,
            },
        );
        true
    }

    pub(super) fn motion_slot(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<&super::motion_evidence::UiCommandMotionAcceptance> {
        let key = self
            .identities
            .get(&UiMountedPresentationIdentityKey::for_identity(identity))?;
        let record = self.commands.get(key)?;
        (record.command.identity() == identity).then_some(&record.motion)
    }

    pub(super) fn inherit_unchanged_motion(&mut self, predecessor: &Self) {
        let inherited = self
            .commands
            .iter()
            .filter_map(|(key, record)| {
                let old = predecessor.commands.get(key)?;
                (old.command.identity() == record.command.identity()
                    && super::command_same_presentation_meaning(&old.command, &record.command))
                .then(|| {
                    (
                        *key,
                        CommandRecord {
                            command: record.command.clone(),
                            motion: old.motion.clone(),
                        },
                    )
                })
            })
            .collect::<Vec<_>>();
        for (key, record) in inherited {
            self.commands.insert(key, record);
        }
    }

    pub(super) fn inherit_rebound_motion(
        &mut self,
        predecessor: &Self,
        affected: worth_ui_host_contract::UiSurfaceBindingGeneration,
        replacement: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) {
        let inherited = self
            .commands
            .iter()
            .filter_map(|(key, record)| {
                let old = predecessor.commands.get(key)?;
                let same = old.command.identity() == record.command.identity()
                    && super::command_same_presentation_meaning_after_binding_replacement(
                        &old.command,
                        &record.command,
                        affected,
                        replacement,
                    );
                same.then(|| {
                    (
                        *key,
                        CommandRecord {
                            command: record.command.clone(),
                            motion: old.motion.clone(),
                        },
                    )
                })
            })
            .collect::<Vec<_>>();
        for (key, record) in inherited {
            self.commands.insert(key, record);
        }
    }
}

impl UiMountedPresentationIdentityKey {
    fn for_identity(identity: UiMountedPaintCommandIdentity) -> Self {
        use std::hash::{Hash, Hasher};
        let mut first = UiMountedPresentationIdentityHasher::new(0xcbf2_9ce4_8422_2325);
        let mut second = UiMountedPresentationIdentityHasher::new(0x6c62_272e_07bb_0142);
        identity.hash(&mut first);
        identity.hash(&mut second);
        Self(first.finish(), second.finish())
    }
}

struct UiMountedPresentationIdentityHasher(u64);

impl UiMountedPresentationIdentityHasher {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }
}

impl std::hash::Hasher for UiMountedPresentationIdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

impl UiMountedPresentationCommandKey {
    fn for_command(command: &UiMountedPaintCommand) -> Self {
        match command {
            UiMountedPaintCommand::FilledRect { .. } => Self {
                family: 0,
                slot: 0,
                collection: None,
            },
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => Self {
                family: 1,
                slot: 0,
                collection: Some(portal_identity_digest(mechanic.portal_identity())),
            },
            UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                let slot = match mechanic.slot() {
                    UiSemanticTextSlot::Value => 0,
                    UiSemanticTextSlot::CollectionValue {
                        selected_field_ordinal,
                    } => selected_field_ordinal.saturating_add(1),
                    UiSemanticTextSlot::Posture => u16::MAX,
                };
                Self {
                    family: 2,
                    slot,
                    collection: mechanic
                        .collection_row()
                        .map(|row| row.correlation_digest()),
                }
            }
        }
    }
}

fn portal_identity_digest(identity: u64) -> [u8; 32] {
    let mut digest = [0_u8; 32];
    for (index, chunk) in digest.chunks_exact_mut(8).enumerate() {
        chunk.copy_from_slice(&identity.rotate_left((index * 13) as u32).to_le_bytes());
    }
    digest
}
