//! Why a home form is absent today, who owns the missing state, and when it returns.

/// The owner whose state must become durable before the deferred item returns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryStateOwner {
    Relational,
    World,
    QueryHost,
}

/// The roadmap point at which a deferred item returns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReturnPoint {
    /// Relational runs on Store.
    RelationalOnStore,
    /// Runtime state above Relational is durable.
    DurableRuntimeState,
}

/// One deferral: the owner of the missing state and its return point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReopenDeferral {
    pub owner: WorthQueryStateOwner,
    pub return_point: WorthQueryReturnPoint,
}

/// The forms a home takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHomeForm {
    Memory,
    At,
}

/// A home form no open can read yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryHomeAbsent {
    pub form: WorthQueryHomeForm,
    pub deferral: WorthQueryReopenDeferral,
}
