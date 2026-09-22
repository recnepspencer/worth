#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeMechanicsCapacities {
    pub retained_commands: u16,
    pub rectangle_commands: u16,
    pub text_commands: u16,
    pub damage_regions: u16,
    pub order_edits: u16,
    pub text_bytes: u32,
    pub readiness_owners: u8,
    pub resource_registry_entries: u8,
    pub causes_per_owner: u8,
    pub ready_owner_slots: u8,
    pub presentation_slots: u8,
    pub readback_slots: u8,
    pub readback_bytes: u32,
}

impl UiNativeMechanicsCapacities {
    /// Shared across every qualified profile: these numbers size arenas that are
    /// backend-neutral, and forking them per profile would force those arenas
    /// onto the heap.
    pub const QUALIFIED: Self = Self {
        retained_commands: 4_096,
        rectangle_commands: 2_048,
        text_commands: 2_048,
        damage_regions: 4_096,
        order_edits: 4_096,
        text_bytes: 1_048_576,
        readiness_owners: 8,
        resource_registry_entries: 32,
        causes_per_owner: 64,
        ready_owner_slots: 8,
        presentation_slots: 2,
        readback_slots: 4,
        readback_bytes: 16_777_216,
    };
}
