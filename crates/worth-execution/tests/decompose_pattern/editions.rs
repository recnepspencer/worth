use super::*;

#[test]
fn changed_back_callback_edition_invalidates_empty_input_reuse() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let first = decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            |_, context| {
                context.checkpoint(1)?;
                Ok::<f64, MapKernelFailure<()>>(1.0)
            },
        )
        .unwrap();
    assert_eq!(first.values, vec![1.0, 1.0]);

    let changed_back = DecomposeKernelEditions {
        back: 2,
        ..EDITIONS
    };
    let second = decompose
        .run(
            None,
            &subdomain_map(&[]),
            changed_back,
            interior,
            interface,
            |_, context| {
                context.checkpoint(1)?;
                Ok::<f64, MapKernelFailure<()>>(2.0)
            },
        )
        .unwrap();
    assert_eq!(second.values, vec![2.0, 2.0]);
    assert!(second.reuse.interface_solve_reused);
    assert_eq!(second.reuse.back_substitutions_reused, 0);
    assert_eq!(second.total_report.charged_work(), 2);
}

#[test]
fn changed_interface_and_interior_editions_invalidate_downstream_stages() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();

    let changed_interface = DecomposeKernelEditions {
        interface: 2,
        ..EDITIONS
    };
    let interface_result = decompose
        .run(
            None,
            &subdomain_map(&[]),
            changed_interface,
            interior,
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                    solution: 10.0,
                    slices: vec![10.0, 10.0],
                })
            },
            back,
        )
        .unwrap();
    assert!(interface_result.interface_report.is_some());
    assert_eq!(interface_result.reuse.back_substitutions_reused, 0);

    let changed_interior = DecomposeKernelEditions {
        interior: 2,
        interface: 3,
        ..changed_interface
    };
    let denied = decompose.run(
        None,
        &subdomain_map(&[]),
        changed_interior,
        interior,
        interface,
        back,
    );
    assert!(matches!(denied, Err(error) if matches!(error.cause,
        DecomposeFailure::Input(DecomposeInputDenial::InteriorKernelCoverageMismatch))));

    let full = decompose
        .run(
            None,
            &subdomain_map(&entries),
            changed_interior,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(full.interior_report.charged_work(), 2);
}
