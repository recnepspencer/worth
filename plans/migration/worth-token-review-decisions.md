# Worth Token Medium/Review Decision Pass

This file records the explicit review decisions applied to the generated replacement matrix after the first pass left `medium` and `review` buckets.

- Matrix rows reviewed/updated: 383
- Remaining `medium`/`review` rows: 0

## Decisions

- Product prose `WORTH` becomes `Worth`.
- Canonical machine namespaces use lowercase `worth.*`, `worth-*`, and `application/vnd.worth.*`.
- Route fixture values such as `searchRoute:WORTH` stay uppercase because they are test/domain data, not product naming.
- Repository URLs containing `/WORTH` stay unchanged because repository-name migration was explicitly out of scope.
- Internal JS/TS sentinel fields preserve sentinel shape but become `__Worth...`.
- Rust constants/env-like symbols such as `WORTH_QUERY_*` and `WORTH_GRAPH_*` stay uppercase.
- Snake-case identifiers and trybuild path text become lowercase `worth_...`.
