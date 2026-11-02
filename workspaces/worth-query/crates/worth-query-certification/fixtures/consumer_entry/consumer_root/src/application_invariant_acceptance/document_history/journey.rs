//! One normal entry composes authored futures, real outputs and disposable views.
use super::*;
use model::{with_branch_request, with_request};
use worth_query_host::facade::declaration::application_history::ApplicationHistoryRecentSteps;

fn describe(document: &Configured<'_>, app: &Runtime) -> view::View {
    with_request(app, |request| view::page(document, request, None).unwrap())
}

fn navigate(
    document: &mut Configured<'_>,
    app: &Runtime,
    context: &WorthQueryHistoryDescriptionContext,
    intent: WorthQueryHistoryNavigationIntent,
    key: u128,
) -> WorthQueryHistoryNavigationCompletion {
    let command = with_request(app, |request| {
        navigation::capture(document, request, context, intent, key)
    })
    .unwrap();
    navigation::complete(document, app, &command)
}

fn values(document: &Configured<'_>, app: &Runtime, y: u64, child: &str, child_y: u64) {
    let selection = with_request(app, |request| {
        let rows = document.read(request).unwrap();
        assert_eq!(rows.rows()[0].value, length(y + 1));
        document.selection(request).unwrap()
    });
    // Read the actual selected composite. The expected root and child values
    // come from the authored inputs and fixture geometry, not history metadata.
    with_branch_request(
        app,
        selection.physical_endpoint().product_branch(),
        |request| {
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(y)
            );
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: format!("final:{child}")
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(child_y)
            );
        },
    );
}

pub(super) fn run(app: &Runtime) {
    let mut document = setup::configure(model::attach(app, "anchor-a"), app);
    let sibling = model::attach(app, "remote-a");
    assert_eq!(
        with_request(app, |request| sibling.read(request).unwrap().rows()[0]
            .value),
        length(42)
    );
    let root = with_request(app, |request| document.current_step(request).unwrap());
    let initial = describe(&document, app);
    let context = initial.context.clone();
    assert_eq!(initial.current, root.id());
    assert!(initial.after.is_none());
    let bad = context
        .clone()
        .for_version(WorthQueryHistoryDescriptionContext::CURRENT_VERSION + 1);
    assert!(matches!(
        document.commands(&bad),
        Err(WorthQueryHistoryCommandContextDenial::UnsupportedVersion { .. })
    ));
    let sibling_context = with_request(app, |request| {
        sibling
            .history_page(request, view::limits(), None)
            .unwrap()
            .context()
            .clone()
    });
    assert_eq!(
        document.commands(&sibling_context).err().unwrap(),
        WorthQueryHistoryCommandContextDenial::ForeignDocument
    );
    let mut authored = Vec::new();
    for (offset, (y, child, child_y)) in [
        (2, "remote-b", 43),
        (3, "sibling-b", 23),
        (5, "sibling-c", 32),
    ]
    .into_iter()
    .enumerate()
    {
        let (next, step) =
            authoring::record(document, app, &context, y, 0x9182_7200 + offset as u64);
        document = next;
        assert_eq!(
            step.parent(),
            Some(
                authored
                    .last()
                    .map_or(root.id(), |step: &WorthQueryHistoryStep| step.id())
            )
        );
        values(&document, app, y, child, child_y);
        authored.push(step);
    }
    let (s1, s2, s3) = (&authored[0], &authored[1], &authored[2]);
    let listed = describe(&document, app);
    assert_eq!(
        listed.rows.iter().map(|row| row.id()).collect::<Vec<_>>(),
        [root.id(), s1.id(), s2.id(), s3.id()]
    );
    assert!(listed
        .rows
        .iter()
        .skip(1)
        .all(|row| row.label() == Some("Set model Y")));
    let preview = with_request(app, |request| {
        document
            .commands(&context)
            .unwrap()
            .preview(request, s1.id())
            .unwrap()
    });
    with_request(app, |request| {
        let read = preview.read(&document, request).unwrap();
        assert_eq!(read.rows()[0].value, length(3));
        assert_eq!(read.source(), s1.endpoint());
        read.close();
    });
    let selected = navigate(
        &mut document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(s1.id()),
        0x9182_7210,
    );
    assert_eq!(selected.selection().step().id(), s1.id());
    assert_eq!(selected.selection().step().endpoint(), s1.endpoint());
    assert_ne!(selected.selection().physical_endpoint(), s1.endpoint());
    values(&document, app, 2, "remote-b", 43);
    let (next, t2) = authoring::record(document, app, &context, 4, 0x9182_7211);
    document = next;
    assert_eq!(t2.parent(), Some(s1.id()));
    assert_ne!(t2.id(), s2.id());
    values(&document, app, 4, "sibling-c", 32);
    let undone = navigate(
        &mut document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::UndoStep,
        0x9182_7212,
    );
    assert_eq!(undone.selection().step().id(), s1.id());
    let choices = with_request(app, |request| {
        document
            .redo_choices(request, view::limits(), None)
            .unwrap()
    });
    assert_eq!(
        choices.choice(),
        WorthQueryHistoryRedoChoice::ChoiceRequired
    );
    assert_eq!(choices.children().context(), &context);
    assert_eq!(
        choices
            .children()
            .rows()
            .iter()
            .map(|row| row.step().id())
            .collect::<Vec<_>>(),
        [s2.id(), t2.id()]
    );
    for (child, y, key, output, expected) in [
        (s2.id(), 3, 0x9182_7220, "sibling-b", 23),
        (t2.id(), 4, 0x9182_7222, "sibling-c", 32),
    ] {
        let done = navigate(
            &mut document,
            app,
            &context,
            WorthQueryHistoryNavigationIntent::RedoTo(child),
            key,
        );
        assert_eq!(done.selection().step().id(), child);
        values(&document, app, y, output, expected);
        navigate(
            &mut document,
            app,
            &context,
            WorthQueryHistoryNavigationIntent::UndoStep,
            key + 1,
        );
    }
    let done = navigate(
        &mut document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(s3.id()),
        0x9182_7230,
    );
    assert_eq!(done.selection().step().id(), s3.id());
    values(&document, app, 5, "sibling-c", 32);
    drop(initial);
    drop(listed);
    drop(choices);
    let rebuilt = describe(&document, app);
    assert_eq!(rebuilt.current, s3.id());
    assert_eq!(rebuilt.context, context);
    let row = rebuilt.rows.iter().find(|row| row.id() == s1.id()).unwrap();
    navigate(
        &mut document,
        app,
        &rebuilt.context,
        WorthQueryHistoryNavigationIntent::Checkout(row.id()),
        0x9182_7231,
    );
    navigate(
        &mut document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(s3.id()),
        0x9182_7232,
    );
    values(&document, app, 5, "sibling-c", 32);
    let group = with_request(app, |request| {
        authoring::group(document, app, request, &context, 0x9182_7260)
    })
    .unwrap_or_else(|(_, cause)| panic!("normal grouped gesture: {cause:?}"));
    let (next, grouped) = authoring::complete_group(group, app);
    document = next;
    assert_eq!(grouped.parent(), Some(s3.id()));
    values(&document, app, 3, "sibling-b", 23);
    with_request(app, |request| {
        document
            .commands(&context)
            .unwrap()
            .bookmark::<PlanarHistoryBookmark<ConsumerSchema>>(request, s1.id(), true)
            .unwrap()
    });
    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    for _ in 0..16 {
        let status = with_request(app, |request| {
            document
                .commands(&context)
                .unwrap()
                .window(target)
                .advance::<PlanarHistoryWindow<ConsumerSchema>>(request)
                .unwrap()
        });
        if status.pending_window.is_none() {
            break;
        }
    }
    with_request(app, |request| {
        let status = document.history_retention_status(request).unwrap();
        assert_eq!(
            (
                status.recent_steps,
                status.pending_window,
                status.unprotected_steps,
                status.bookmarks
            ),
            (target, None, 1, 1)
        );
        assert_eq!(
            preview.read(&document, request).unwrap().rows()[0].value,
            length(3)
        );
    });
    preview.close();
    with_request(app, |request| {
        document
            .commands(&context)
            .unwrap()
            .bookmark::<PlanarHistoryBookmark<ConsumerSchema>>(request, s1.id(), false)
            .unwrap()
    });
    for _ in 0..64 {
        if !with_request(app, |request| {
            document
                .history_retention_status(request)
                .unwrap()
                .maintenance_pending()
        }) {
            break;
        }
        with_request(app, |request| {
            document
                .commands(&context)
                .unwrap()
                .maintain(request)
                .unwrap()
        });
    }
    assert_eq!(
        with_request(app, |request| sibling.read(request).unwrap().rows()[0]
            .value),
        length(42)
    );
    drop(rebuilt);
    model::close(document, app);
    model::close(sibling, app);
    assert!(app.branches().pending_cleanup().is_empty());
}
