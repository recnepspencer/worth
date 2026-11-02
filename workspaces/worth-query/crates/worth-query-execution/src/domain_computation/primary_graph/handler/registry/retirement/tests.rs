//! Local original-owner mechanics, not service or physical close authority.
use super::*;
use crate::domain_computation::primary_graph::{
    application_entry::mutation::{CandidateWriter, DecisionReader, HandlerResult},
    tests::fixture::{
        CapabilityTouchMutationBinding as Binding, IdentityExecutionSchema as Schema,
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};
type Input = <Binding as ApplicationMutationBinding<Schema>>::Input;
type Decision = <Binding as ApplicationMutationBinding<Schema>>::Decision;
type DomainDenial = <Binding as ApplicationMutationBinding<Schema>>::Denial;
type Output = <Binding as ApplicationMutationBinding<Schema>>::Result;

struct Probe {
    contacts: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    refuses: bool,
}
impl Drop for Probe {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl OperationHandler<Schema, Binding> for Probe {
    fn preflight_home_retirement(&mut self, _: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        if self.refuses {
            Err(Denial::Unsupported)
        } else {
            Ok(())
        }
    }
    fn advance_home_retirement(
        &mut self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        if self.refuses {
            Err(Denial::Unsupported)
        } else {
            Ok(Progress::Complete)
        }
    }
    fn decide(
        &self,
        _: &Input,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, DomainDenial> {
        panic!("retirement does not execute business")
    }
    fn build_candidate(
        &self,
        _: &Input,
        _: Decision,
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Output, DomainDenial> {
        panic!("retirement does not build a candidate")
    }
}
fn entry(contacts: &Arc<AtomicUsize>, drops: &Arc<AtomicUsize>) -> PendingMutationHandler {
    PendingMutationHandler::retain_original::<Schema, Binding, Probe>(
        Probe {
            contacts: contacts.clone(),
            drops: drops.clone(),
            refuses: false,
        },
        HandlerOrigin::Supplier,
    )
}

#[test]
fn original_erased_slot_and_handler_aliases_preserve_the_same_handler_and_payload() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let contacts = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut value = entry(&contacts, &drops).value;
    let original = Arc::as_ptr(&value);
    let outer = value.clone();
    assert!(matches!(
        preflight_original_handler::<Schema, Binding>(&mut value, &request),
        Err(Denial::Busy)
    ));
    drop(outer);
    let outer = Arc::downgrade(&value);
    assert!(matches!(
        preflight_original_handler::<Schema, Binding>(&mut value, &request),
        Err(Denial::Busy)
    ));
    drop(outer);
    let inner = Arc::get_mut(&mut value)
        .unwrap()
        .downcast_mut::<TypedHandlerSlot<Schema, Binding>>()
        .unwrap()
        .handler
        .clone();
    assert!(matches!(
        preflight_original_handler::<Schema, Binding>(&mut value, &request),
        Err(Denial::Busy)
    ));
    drop(inner);
    let inner = Arc::downgrade(
        &Arc::get_mut(&mut value)
            .unwrap()
            .downcast_mut::<TypedHandlerSlot<Schema, Binding>>()
            .unwrap()
            .handler,
    );
    assert!(matches!(
        preflight_original_handler::<Schema, Binding>(&mut value, &request),
        Err(Denial::Busy)
    ));
    drop(inner);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(std::ptr::addr_eq(original, Arc::as_ptr(&value)));
    preflight_original_handler::<Schema, Binding>(&mut value, &request).unwrap();
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(value);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn ordinary_supplier_retirement_retains_payload_with_an_exhausted_memory_budget() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let contacts = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut registry = InstalledMutationHandlerRegistry::<Schema> {
        entries: BTreeMap::from([(Binding::IDENTITY.to_owned(), entry(&contacts, &drops))]),
        _schema: PhantomData,
    };
    registry.preflight_home_retirement(&request).unwrap();
    let mut cursor = HandlerRetirementCursor::default();
    assert!(matches!(
        registry.advance_home_retirement(&mut cursor, &request),
        Ok(Progress::Advanced)
    ));
    assert!(matches!(
        registry.advance_home_retirement(&mut cursor, &request),
        Ok(Progress::Complete)
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(registry);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn supplier_declared_refusal_preserves_original_handler_and_payload() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let contacts = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let entry = PendingMutationHandler::retain_original::<Schema, Binding, Probe>(
        Probe {
            contacts: contacts.clone(),
            drops: drops.clone(),
            refuses: true,
        },
        HandlerOrigin::Supplier,
    );
    let original = Arc::as_ptr(&entry.value);
    let mut registry = InstalledMutationHandlerRegistry::<Schema> {
        entries: BTreeMap::from([(Binding::IDENTITY.to_owned(), entry)]),
        _schema: PhantomData,
    };
    assert!(matches!(
        registry.preflight_home_retirement(&request),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Unsupported
        })
    ));
    assert!(matches!(
        registry.advance_home_retirement(&mut HandlerRetirementCursor::default(), &request),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Unsupported
        })
    ));
    assert!(std::ptr::addr_eq(
        original,
        Arc::as_ptr(&registry.entries[Binding::IDENTITY].value)
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(registry);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

struct Ordinary(Arc<AtomicUsize>);
impl OperationHandler<Schema, Binding> for Ordinary {
    fn decide(
        &self,
        _: &Input,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, DomainDenial> {
        panic!("retirement cannot invoke the handler")
    }
    fn build_candidate(
        &self,
        _: &Input,
        _: Decision,
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Output, DomainDenial> {
        panic!("retirement cannot invoke the handler")
    }
}
impl Drop for Ordinary {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn default_supplier_hooks_retire_invocation_without_destroying_shared_payload() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let drops = Arc::new(AtomicUsize::new(0));
    let mut value = PendingMutationHandler::retain_original::<Schema, Binding, Ordinary>(
        Ordinary(drops.clone()),
        HandlerOrigin::Supplier,
    )
    .value;
    preflight_original_handler::<Schema, Binding>(&mut value, &request).unwrap();
    assert!(matches!(
        advance_original_handler::<Schema, Binding>(&mut value, &request),
        Ok(Progress::Complete)
    ));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(value);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

use crate::domain_computation::primary_graph::{
    tests::fixture::{self, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};
#[test]
fn erased_slot_and_active_handler_aliases_refuse_while_the_same_query_serves() {
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = fixture::installed_authorization_world(true);
    let scope = fixture::live_scope();
    let original =
        Arc::as_ptr(&world.application.mutation_handlers.entries[Binding::IDENTITY].value);
    let alias = world.application.mutation_handlers.entries[Binding::IDENTITY]
        .value
        .clone();
    assert!(matches!(
        world
            .application
            .mutation_handlers
            .preflight_home_retirement(&execution),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Busy
        })
    ));
    let resolve = |world: &fixture::AuthorizationWorld| {
        world
            .application
            .with_host_advancement(|phase| {
                world
                    .selected_product(&phase.test_host_request())
                    .resolve_entity(
                        &phase,
                        AccountStatus::reference(),
                        "open".to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .expect("the same original Query resolves its account after refusal")
            })
            .unwrap()
    };
    let before = resolve(&world);
    drop(alias);
    let alias = world
        .application
        .mutation_handler_for_attempt::<Binding>()
        .unwrap();
    assert!(matches!(
        world
            .application
            .mutation_handlers
            .preflight_home_retirement(&execution),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Busy
        })
    ));
    let after = resolve(&world);
    assert_eq!(before.entity_id(), after.entity_id());
    assert!(std::ptr::addr_eq(
        original,
        Arc::as_ptr(&world.application.mutation_handlers.entries[Binding::IDENTITY].value)
    ));
    drop(alias);
    world
        .application
        .mutation_handlers
        .preflight_home_retirement(&execution)
        .unwrap();
    let final_observation = resolve(&world);
    assert_eq!(before.entity_id(), final_observation.entity_id());
}
