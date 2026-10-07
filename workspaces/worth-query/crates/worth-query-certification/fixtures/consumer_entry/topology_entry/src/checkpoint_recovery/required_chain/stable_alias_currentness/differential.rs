//! Seeded source edits compare real retained output against a plain input model.

use super::*;
use worth_query_consumer_values::PositiveLength;

#[test]
fn seeded_native_source_edits_agree_with_plain_model_and_full_output_verification() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in [0x9176_3a11_u64, 0x9176_3b22] {
        // These 24-step seeds publish source edits, changed producer outputs,
        // and one current_output verification per step. Their real ancestor
        // history exceeds the ordinary 32-commit checkpoint fixture budget.
        // Each exact historical root retains its own conservative invalidation
        // capacity bound, even where physical nodes are shared. This journey
        // grants a finite 512 MiB ledger for its 64 history positions.
        let application = support::install_program_with_history::<program::ChainProgram>(
            None,
            Default::default(),
            64,
            512 * 1_024 * 1_024,
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut model_y = ["anchor-a", "anchor-b"].map(|body| {
            let selected = request
                .query(PlanarRead {
                    body_key: body.to_owned(),
                })
                .execute()
                .unwrap();
            PositiveLength::get(&selected.rows()[0].y)
        });
        let initial = settle_anchor(&request, &application, seed, 0);
        assert_eq!(initial.producer_contacts_in_this_demand(), 1);
        assert_eq!(
            initial.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        let mut random = seed;
        let mut cases = [0; 3];

        for step in 0..24_u64 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let case = (random % 3) as usize;
            cases[case] += 1;
            let changes_input = case == 2;
            let idempotency = seed.wrapping_add(100 + 2 * step);
            if case != 0 {
                let index = usize::from(case == 1);
                let body = ["anchor-a", "anchor-b"][index];
                let mut replacement = 2 + random % 7;
                if replacement == model_y[index] {
                    replacement += 1;
                }
                let source = request
                    .query(PlanarRead {
                        body_key: body.to_owned(),
                    })
                    .execute()
                    .unwrap();
                request
                    .mutate(PlanarSourceAdjustment {
                        scope_key: body.to_owned(),
                        replacement_y: length(replacement),
                    })
                    .expect_source(source.observed_sources()[0].clone())
                    .idempotency(&idempotency)
                    .execute_performed::<program::ChainProgram, program::ChainRoot>(&application)
                    .expect("the selected source edit commits through the real mutation owner");
                model_y[index] = replacement;
            }

            let before = request.retain_read().unwrap();
            let settled = settle_anchor(&request, &application, seed, step + 1);
            let after = request.retain_read().unwrap();
            assert_eq!(
                settled.producer_contacts_in_this_demand(),
                usize::from(changes_input),
                "seed {seed:x}, step {step}, case {case}",
            );
            assert_eq!(
                before.selected_commit() != after.selected_commit(),
                changes_input,
                "only a changed producer input may require a new performed commit",
            );
            if changes_input {
                assert_eq!(
                    settled.posture(),
                    WorthQueryOutputSettlementPosture::Performed
                );
                assert!(settled.application_commit_receipt().is_some());
            } else if case == 1 {
                assert_eq!(
                    settled.posture(),
                    WorthQueryOutputSettlementPosture::StableReused
                );
                assert!(settled.application_commit_receipt().is_none());
            }
            let output = request
                .query(PlanarOutputRead {
                    body_key: "anchor-a".to_owned(),
                })
                .execute()
                .unwrap();
            assert_eq!(
                PositiveLength::get(&output.rows()[0].value),
                model_y[0] + 1,
                "the independent input model agrees with the actual native output",
            );
            let verified = mutate_anchor(&request, &application, output_probe(), idempotency + 1);
            assert!(
                matches!(
                    &verified,
                    Ok(WorthQueryApplicationMutationOutcome::Committed { .. }),
                ),
                "seed {seed:x}, step {step}: current_output certifies the actual output: {verified:?}",
            );
        }
        assert!(cases.into_iter().all(|count| count > 0));
    }
}

fn settle_anchor<'application>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
    application: &'application application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    seed: u64,
    progression: u64,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    PlanarQuery,
> {
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(application)
        .expect("the installed ordinary demand selects its actual source");
    (0..256)
        .find_map(|_| {
            match demand.advance(request).unwrap_or_else(|denial| {
                panic!("seed {seed:x}, progression {progression}: {denial:?}")
            }) {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the installed ordinary demand settles within its bounded progression")
}
