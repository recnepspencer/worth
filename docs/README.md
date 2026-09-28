# WORTH Documentation

This directory holds the public documentation for the WORTH platform. Start
here to find the right document for your question.

Internal milestone plans live in [`plans/`](../plans/). They record what the
team intends to build and what is complete. They are not consumer
documentation.

## Reading order

Read these in order the first time. Keep the Glossary open as a reference while you read the others.

| # | Document | Answers |
|---|---|---|
| 1 | [Platform README](../README.md) | What is WORTH? Which crates exist, and what does each one do? |
| 2 | [Philosophy](philosophy.md) | Why is the platform shaped this way? Why are truth, state, and authority kept apart? |
| 3 | [How WORTH Works](how-it-works.md) | What happens to a request, from declaration to publication? What do the runtimes guarantee? |
| 4 | [Glossary](glossary.md) | What exactly does a term such as *basis*, *admission*, *performed*, or *settled* mean? |
| 5 | [API Map](api.md) | Which crate do I import for my job? Where is the reference documentation? |

## Find a document by task

| I want to... | Read |
|---|---|
| Understand the platform in five minutes | [Platform README](../README.md) |
| Understand the reasons behind a rule | [Philosophy](philosophy.md) |
| Build an application on WORTH | [API Map](api.md), then the Query crate guides it links |
| Understand what the runtimes guarantee to my code | [How WORTH Works](how-it-works.md) |
| Look up a term | [Glossary](glossary.md) |
| Contribute code to the platform | [Coding Guidelines](coding-guidelines/) and [AGENTS.md](../AGENTS.md) |
| See what is planned or in progress | [`plans/`](../plans/) |

## Coding guidelines

The engineering laws that bind every change to the platform:

| Document | Scope |
|---|---|
| [MENTALITY.md](coding-guidelines/MENTALITY.md) | The foundational mindset: build order, enforcement, authority before derivation |
| [arch_laws.md](coding-guidelines/arch_laws.md) | Facades, typed phase progression, envelopes, precedence |
| [composition_laws.md](coding-guidelines/composition_laws.md) | One named responsibility per file |
| [domain_structure_laws.md](coding-guidelines/domain_structure_laws.md) | Physical boundaries preserve meaning and authority |
| [perf_laws.md](coding-guidelines/perf_laws.md) | Hot-path honesty and carried proof |
| [testing_laws.md](coding-guidelines/testing_laws.md) | Fixture honesty and real boundaries |
| [dx_laws.md](coding-guidelines/dx_laws.md) | Developer experience as a real code target |
| [qa_review_guide.md](coding-guidelines/qa_review_guide.md) | How changes are reviewed |

## Notes for AI agents

- Read in this order: [Philosophy](philosophy.md), then
  [How WORTH Works](how-it-works.md) (its section 17 is a summary for you),
  then [API Map §1](api.md#1-the-rule-in-one-sentence).
- Treat [Philosophy](philosophy.md) and [How WORTH Works](how-it-works.md) as
  the platform's mental model. Crate guides add API detail but do not change
  that model.
- Every Rust name in these documents exists in the current source. If a name
  you need is not on an audience facade listed in the [API Map](api.md), do not
  import it.
- Definitions in the [Glossary](glossary.md) are normative. When a crate guide
  and the glossary disagree, report the disagreement.
- `plans/` describes intent. Do not treat a plan as proof that a feature exists.
