//! Moved whole from `domain_computation::primary_graph::application_contribution::computation::wave_dispatch::seeded_dispatch_orders`: the items below and every impl they own.
use std::cell::RefCell;

std::thread_local! {
    pub(in crate::domain_computation::primary_graph) static ORDERS: RefCell<Option<Vec<Vec<usize>>>> = const { RefCell::new(None) };
}

pub(in crate::domain_computation::primary_graph) fn record(order: Vec<usize>) {
    ORDERS.with(|orders| {
        if let Some(orders) = orders.borrow_mut().as_mut() {
            orders.push(order);
        }
    });
}
