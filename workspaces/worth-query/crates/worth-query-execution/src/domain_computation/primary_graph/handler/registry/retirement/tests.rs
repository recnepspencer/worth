//! Local original-owner mechanics, not service or physical close authority.
use super::*;
mod retirement_binding;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerResult,
};
use retirement_binding::{Binding, Schema};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};
type Input = <Binding as ApplicationMutationBinding<Schema>>::Input;
type Decision = <Binding as ApplicationMutationBinding<Schema>>::Decision;
type DomainDenial = <Binding as ApplicationMutationBinding<Schema>>::Denial;
type Output = <Binding as ApplicationMutationBinding<Schema>>::Result;

struct PanickingPayload;
impl Drop for PanickingPayload {
    fn drop(&mut self) {
        panic!("supplier panic payload destructor");
    }
}
struct Probe {
    contacts: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    refuses: bool,
    panic_once: Arc<AtomicBool>,
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
        if self.panic_once.swap(false, Ordering::SeqCst) {
            std::panic::panic_any(PanickingPayload);
        }
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
            panic_once: Arc::new(AtomicBool::new(false)),
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
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    let contacts = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut registry = InstalledMutationHandlerRegistry::<Schema> {
        entries: BTreeMap::from([(Binding::IDENTITY.to_owned(), entry(&contacts, &drops))]),
        _schema: PhantomData,
    };
    registry.preflight_home_retirement(&request).unwrap();
    let exhausted = budget
        .reserve((1 << 20) - budget.charged_memory_bytes())
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 1 << 20);
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
    drop(exhausted);
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
            panic_once: Arc::new(AtomicBool::new(false)),
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

#[test]
fn handler_retirement_and_panic_payload_destructor_preserve_original_custody_for_retry() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
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
            refuses: false,
            panic_once: Arc::new(AtomicBool::new(true)),
        },
        HandlerOrigin::Supplier,
    );
    let original = Arc::as_ptr(&entry.value);
    let mut registry = InstalledMutationHandlerRegistry::<Schema> {
        entries: BTreeMap::from([(Binding::IDENTITY.to_owned(), entry)]),
        _schema: PhantomData,
    };
    let mut cursor = HandlerRetirementCursor::default();
    assert!(matches!(
        registry.advance_home_retirement(&mut cursor, &request),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Execution(worth_execution::WorkCeilingDenial::Panicked)
        })
    ));
    assert!(cursor.completed.is_none());
    assert!(std::ptr::addr_eq(
        original,
        Arc::as_ptr(&registry.entries[Binding::IDENTITY].value)
    ));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
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
