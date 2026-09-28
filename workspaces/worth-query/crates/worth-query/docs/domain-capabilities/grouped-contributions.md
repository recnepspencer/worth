# Grouped Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

Grouped contributions are for neighborhoods that need both shared group
posture and member-local contribution authoring.

Use them when:

- one contribution should apply to the whole neighborhood
- another contribution should only apply to one member
- you want to keep that distinction visible in the retained result

Build them from the grouped declaration input:

- `with_shared_support_contribution(...)`
- `with_shared_explanation_contribution(...)`
- `with_shared_workflow_contribution(...)`
- `with_member_contribution(member_index, ...)`

Then run:

- `grouped_contributions_checked(...)`
- `grouped_contributions_for_active_face_selection_checked(...)`

The grouped surface still lowers through the canonical
contribution-composed orchestration engine. The grouped layer just preserves:

- which contributions were shared
- which contributions were member-local
- which member owned the resulting composed artifact or stop

This is the right surface when a neighborhood edit needs shared explanation,
shared support, or shared workflow posture without flattening every
contribution into one undifferentiated bag.
