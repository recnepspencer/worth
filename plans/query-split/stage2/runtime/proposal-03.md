# Proposal 03 — context types that hand out the runtime

Status: option A is the path ruling (a) already names for public context types ("a public
type made generic gets its composed alias in application_runtime"). It is taken after the
phase-2 milestone validation, and recorded here because it changes how two public types are
spelled. Everything below the runtime is converted; 33 temporary `application_runtime()`
bridges remain, and nearly all of them come from the problem below.

## The problem

Under ruling (a), context types hold `&'r dyn <Module>Host<Schema>`. They already do:
`WorthQuerySelectedProductOperation` holds `&'r dyn ProductOperationHost<Schema>`, and the
same goes for `WorthQueryDocumentHistory`, the branch heads and the correction source. Two
things break once the bridges go.

1. **Higher modules' methods on a lower context.** `WorthQuerySelectedProductOperation` has
   65 impl blocks spread over 9 modules (proposal 01, summary map). The query-admission
   blocks in `product_operation::query`, the conditional-clock blocks in
   `conditional_operation`, the correction disclosure and the history navigation all need a
   host higher than the one the context holds: the application query host, the conditional
   operation host, or installation's selected program owner. A `&dyn ProductOperationHost`
   cannot become a `&dyn ConditionalOperationHost` without a downcast or an upward method
   naming the higher trait. Either one is an upward edge.
2. **Public accessors that hand out the concrete runtime.**
   `WorthQueryDocumentHistory::application()` is public, and `worth-query-publication` calls
   it at 45 sites. Publication uses the returned runtime to open advancements, to build
   `WorthQueryApplicationRequest { application, .. }` (which holds `&Runtime`) and for
   `ptr::eq` ownership checks. `WorthQueryProgramApplicationRuntime::runtime()` and its
   `Deref` are the same shape. A dyn-holding type cannot return `&Runtime`.

## Options

**A. Generic host on the public context types, with composed aliases (recommended).** This
is the path the ruling already names: "a public type made generic gets its composed alias in
application_runtime".
- Add a defaulted parameter:
  `WorthQuerySelectedProductOperation<'r, S, H: ?Sized + 'r = dyn ProductOperationHost<S> + 'r>`
  holding `&'r H`. `WorthQueryDocumentHistory` and the branch-head and correction contexts get
  the same parameter.
- Each module's impl blocks become `impl<.., H: <ThatModule>Host<S> + ?Sized>`, so they can
  move to the modules that own them (brief step 3), and `application()` returns `&'r H`.
- Extension methods that construct a context return it typed by `Self`. Called on the runtime
  through a facade, they give consumers a runtime-typed context. Internal callers holding a
  `dyn` host get the default.
- Consumers keep writing `WorthQuerySelectedProductOperation<'r, S>` through the facade alias
  `= product_operation::WorthQuerySelectedProductOperation<'r, S, WorthQueryPrimaryGraphApplicationRuntime<S>>`.
  Internal code naming the struct keeps the `dyn` default.
- Cost: about 270 mentions in the library, 25 publication files and 9 testkit files need
  re-checking. One erase step (`&H` to `&dyn`) is needed wherever a runtime-typed context is
  handed to an internal function that takes the default. Each generic is recorded as the
  ruling requires.

**B. Change the public accessors.** `application()` becomes ownership checks
(`belongs_to(&Runtime)`), and publication takes the runtime from its request. This is a
behaviour-visible change in a sibling crate: about 25 read paths have no request runtime
today and would need it threaded through. It doesn't solve problem 1.

**C. A checked downcast** (`Any` on the identity host). This is outside the rulings and trades
a compile-time boundary for a runtime check. Not recommended.

## What proceeds without the ruling

The other upward calls (attempt to product operation, handler to product operation, provider
to attempt, invariant projection to contribution, and authorization to attempt, which is
step 5's contract) are host-method work. The data-type pairs from the ownership sheet are
script moves. Neither depends on this choice.
