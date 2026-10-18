# Feature Doc QA Checklist

Do not ship the doc until these questions all have a good answer.

## Boundary Honesty

- Does the doc describe the actual stable public surface?
- Does it clearly separate stable, deferred, unsupported, and
  vocabulary-only surfaces?
- Does it preserve lower-runtime authority boundaries?

## Learnability

- Could a capable engineer use the feature after reading only this doc and the
  adjacent linked docs?
- Does the doc explain the mental model before details?
- Is there a smallest honest example?
- Is there a realistic example that touches adjacent features?

## Correctness

- Are lifecycle, phase, or execution semantics explained where needed?
- Are anti-patterns called out explicitly?
- Are caveats placed near the code that needs them?
- Does the doc avoid promising behavior that only exists in roadmap or
  milestone prose?

## AI Usability

- Could an AI implement against this doc without spelunking tests?
- Does the doc name the correct public entry points precisely?
- Does it explain what not to do, not just what to do?

## WORTH Quality Bar

- Does this read like product documentation rather than an engineering spec?
- Does it avoid giant option dumps and milestone archaeology?
- Would this still be a good reference six months from now?
