use super::*;

async fn assert_other_operation_survives_repeated_blocked_page(
    repeated_operation: usize,
    later_operation: usize,
) {
    let pages = (0..26).map(|page| {
        let late = page == 25;
        let mut pools = [BankRailMaintenancePool::default(); 2];
        pools[repeated_operation] = BankRailMaintenancePool {
            signed_available_before: 4,
            signed_selected: 4,
            ..Default::default()
        };
        pools[later_operation] = BankRailMaintenancePool {
            signed_available_before: 104,
            signed_selected: 4,
            ..Default::default()
        };
        BankRailMaintenanceBatch {
            selected: 8,
            pools,
            blocked: if late { 7 } else { 8 },
            performed: usize::from(late),
            remaining: if late { 0 } else { 108 },
            ..Default::default()
        }
    });
    let route = Arc::new(ScriptedRoute::with_batches(pages));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 26).await;
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

#[tokio::test]
async fn blocked_estate_signed_page_does_not_consume_payment_signed_budget() {
    assert_other_operation_survives_repeated_blocked_page(0, 1).await;
}

#[tokio::test]
async fn blocked_payment_signed_page_does_not_consume_estate_signed_budget() {
    assert_other_operation_survives_repeated_blocked_page(1, 0).await;
}

async fn assert_disappeared_pool_is_parked(disappeared: usize, blocked_peer: usize) {
    let mut first = [BankRailMaintenancePool::default(); 2];
    first[disappeared] = BankRailMaintenancePool {
        signed_available_before: 8,
        signed_selected: 4,
        ..Default::default()
    };
    first[blocked_peer] = BankRailMaintenancePool {
        signed_available_before: 1,
        signed_selected: 1,
        ..Default::default()
    };
    let mut second = [BankRailMaintenancePool::default(); 2];
    // A failed or vanished operation reports no available work. The peer
    // remains blocked and selected, so it must not drain the vanished count.
    second[blocked_peer] = first[blocked_peer];
    let route = Arc::new(ScriptedRoute::with_batches([
        BankRailMaintenanceBatch {
            selected: 5,
            pools: first,
            blocked: 5,
            remaining: 5,
            ..Default::default()
        },
        BankRailMaintenanceBatch {
            selected: 1,
            pools: second,
            blocked: 1,
            remaining: 1,
            ..Default::default()
        },
    ]));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 2).await;
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(
        route.calls.load(Ordering::Acquire),
        2,
        "vanished pool parks without another batch"
    );
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

#[tokio::test]
async fn vanished_estate_pool_parks_while_payment_peer_is_blocked() {
    assert_disappeared_pool_is_parked(0, 1).await;
}

#[tokio::test]
async fn vanished_payment_pool_parks_while_estate_peer_is_blocked() {
    assert_disappeared_pool_is_parked(1, 0).await;
}
