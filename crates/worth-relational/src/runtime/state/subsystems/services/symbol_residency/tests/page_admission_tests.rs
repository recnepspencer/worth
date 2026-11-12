//! Cold dictionary page admission, sticky damage and retryable source failures.
use super::*;

#[test]
fn pending_symbols_refuse_a_cold_bucket_before_issuing_an_identity() {
    with_symbol_dictionary_world(
        |SymbolDictionaryWorld {
             table,
             source,
             all: snapshot,
             execution,
             ..
         }| {
            use crate::runtime::state::subsystems::services::symbol_table::SymbolReservationDenial;

            let (existing, value) = snapshot.entries.last().unwrap();
            let before = table.interner_snapshot();
            let mut candidate = table.begin_pending_meanings();
            assert!(matches!(
                candidate.reserve(value.as_str()),
                Err(SymbolReservationDenial::ColdBucket(_))
            ));
            assert_eq!(table.interner_snapshot(), before);
            assert_eq!(source.reads.load(Ordering::SeqCst), 0);

            table.demand_text(&execution, value.as_str()).unwrap();
            assert_eq!(candidate.reserve(value.as_str()).unwrap(), *existing);
            let meanings = candidate.interner().snapshot();
            {
                let binding = candidate.binding();
                candidate.seal_for_publication(&meanings, &binding, [])
            }
            .expect("fixture meanings agree with the candidate")
            .publish()
            .unwrap();
        },
    );
}

#[test]
fn id_and_value_demands_load_pages_and_preserve_interning_identity_and_frontier() {
    with_symbol_dictionary_world(
        |SymbolDictionaryWorld {
             table,
             directory,
             source,
             all,
             execution,
             ..
         }| {
            assert!(directory.manifest.symbols.len() > 1);
            assert!(table.snapshot().entries.is_empty());
            assert_eq!(source.reads.load(Ordering::SeqCst), 0);

            let first = all
                .entries
                .iter()
                .min_by_key(|(symbol, _)| *symbol)
                .unwrap();
            table.demand_symbol_ids(&execution, [first.0]).unwrap();
            assert!(table.with_read(|symbols| symbols.resolve(first.0) == Some(first.1.as_str())));
            assert!(source.reads.load(Ordering::SeqCst) < 3);
            assert!(table.snapshot().entries.len() < all.entries.len());
            let last = all
                .entries
                .iter()
                .max_by_key(|(symbol, _)| *symbol)
                .unwrap();
            assert!(table.with_read(|symbols| symbols.symbol(last.1.as_str()).is_none()));
            let before = source.reads.load(Ordering::SeqCst);
            table.demand_text(&execution, last.1.as_str()).unwrap();
            assert_eq!(
                table.with_write(|symbols| symbols.intern(last.1.as_str())),
                Ok(last.0)
            );
            assert!(source.reads.load(Ordering::SeqCst) - before < 3);
            table
                .demand_text(&execution, "new-after-cold-directory")
                .unwrap();
            let issued = table.with_write(|symbols| {
                symbols
                    .intern("new-after-cold-directory")
                    .expect("symbol fixture has identity capacity")
            });
            assert!(
                issued.0 > last.0 .0,
                "partial dictionary admission must not reissue an old id"
            );
        },
    );
}

#[test]
fn a_damaged_symbol_page_is_a_sticky_typed_refusal() {
    with_symbol_dictionary_world(
        |SymbolDictionaryWorld {
             table,
             source,
             all,
             execution,
             ..
         }| {
            let symbol = all.entries[0].0;
            source.corrupt.store(true, Ordering::SeqCst);
            let first = table.demand_symbol_ids(&execution, [symbol]).unwrap_err();
            assert!(matches!(&first, Failure::StorageUnavailable(failure)
            if matches!(failure.subject, RelationalStorageSubject::Symbols { .. }) && failure.cause == Cause::IntegrityMismatch));
            source.corrupt.store(false, Ordering::SeqCst);
            let reads = source.reads.load(Ordering::SeqCst);
            assert_eq!(
                table.demand_symbol_ids(&execution, [symbol]).unwrap_err(),
                first
            );
            assert_eq!(source.reads.load(Ordering::SeqCst), reads);
            assert!(table.snapshot().entries.is_empty());
        },
    );
}

#[test]
fn allocation_failure_in_a_symbol_page_is_retryable() {
    retryable_page_failure(Cause::MemoryExhausted);
}

#[test]
fn transient_source_failure_in_a_symbol_page_is_retryable() {
    retryable_page_failure(Cause::SourceUnavailable);
}

fn retryable_page_failure(cause: Cause) {
    with_symbol_dictionary_world(
        |SymbolDictionaryWorld {
             table,
             source,
             all,
             execution,
             ..
         }| {
            let symbol = all.entries[0].0;
            *source.failure.lock().unwrap() = Some(cause);
            let first = table.demand_symbol_ids(&execution, [symbol]).unwrap_err();
            assert!(matches!(first, Failure::StorageUnavailable(f) if f.cause == cause));
            *source.failure.lock().unwrap() = None;
            table.demand_symbol_ids(&execution, [symbol]).unwrap();
            assert!(table.with_read(|symbols| symbols.retained_text(symbol).is_some()));
        },
    );
}
