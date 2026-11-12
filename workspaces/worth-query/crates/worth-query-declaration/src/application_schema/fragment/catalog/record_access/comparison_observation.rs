//! Test-only comparison observation confined to actual record membership insertion.
use super::ReadTarget;
use std::cell::Cell;

#[derive(Clone, Copy)]
pub(super) enum Owner {
    Local,
    Global,
}
#[derive(Clone, Copy, Default)]
pub(super) struct Counts {
    pub local: u64,
    pub global: u64,
}
#[derive(Clone, Copy, Default)]
struct State {
    active: Option<Owner>,
    recording: bool,
    counts: Counts,
}
thread_local! {
    static STATE: Cell<State> = Cell::new(State::default());
}
pub(super) struct Scope(Option<Owner>);
impl Drop for Scope {
    fn drop(&mut self) {
        STATE.with(|state| {
            state.update(|mut value| {
                value.active = self.0;
                value
            })
        });
    }
}
pub(super) fn enter(owner: Owner) -> Scope {
    STATE.with(|state| {
        let mut value = state.get();
        let previous = value.active;
        value.active = Some(owner);
        state.set(value);
        Scope(previous)
    })
}
pub(super) fn comparison() {
    STATE.with(|state| {
        state.update(|mut value| {
            if value.recording {
                match value.active {
                    Some(Owner::Local) => value.counts.local += 1,
                    Some(Owner::Global) => value.counts.global += 1,
                    None => {}
                }
            }
            value
        })
    });
}
pub(super) struct Workload(State);
impl Workload {
    pub fn start() -> Self {
        STATE.with(|state| {
            Self(state.replace(State {
                recording: true,
                ..State::default()
            }))
        })
    }
    pub fn snapshot(&self) -> Counts {
        STATE.with(|state| state.get().counts)
    }
}
impl Drop for Workload {
    fn drop(&mut self) {
        STATE.with(|state| state.set(self.0));
    }
}

// Exhaustive view preserves every original equality axis; Ord remains derived.
#[derive(PartialEq)]
enum ReadView<'a> {
    ManagedIndex(&'a crate::application_query::ApplicationQueryBindingReference),
    Entity(&'a str),
    Field(&'a str, &'a str, &'a str),
    Relation(&'a str, &'a str, &'a str),
}
fn view(target: &ReadTarget) -> ReadView<'_> {
    match target {
        ReadTarget::ManagedIndex { parent } => ReadView::ManagedIndex(parent),
        ReadTarget::Entity { entity } => ReadView::Entity(entity),
        ReadTarget::Field {
            entity,
            aspect,
            field,
        } => ReadView::Field(entity, aspect, field),
        ReadTarget::Relation { relation, from, to } => ReadView::Relation(relation, from, to),
    }
}
impl PartialEq for ReadTarget {
    fn eq(&self, other: &Self) -> bool {
        comparison();
        view(self) == view(other)
    }
}
