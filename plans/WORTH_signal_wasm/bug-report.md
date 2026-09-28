Findings

ScopedSignalNamespace types advertise computedSpec / outputSpec, but the actual scoped runtime does not implement them.

Evidence:

Types say they exist:

callable_surface.d.ts (line 560)

callable_surface.d.ts (line 563)




Actual scoped runtime object omits them:

scopes.js (line 173)







What I verified:


typeof signals.scope('audit').computedSpec === 'undefined'

typeof signals.scope('audit').outputSpec === 'undefined'


That is a straight type/runtime contradiction.

Nested controller scopes expose computedSpec / outputSpec, but they forward to missing methods and crash.

Evidence:

controller surface forwards directly:

controllers.js (line 94)

controllers.js (line 100)




but nested namespace.scope(...) returns the scoped runtime from scopes.js, which does not have those methods




What I verified:


signals.controller((surface) => surface.scope('nested').computedSpec(...))

throws:

TypeError: namespace.computedSpec is not a function




same for outputSpec


So the package is internally promising a controller authoring surface that its nested scope implementation cannot satisfy.

Scoped input(...) is ambiguous for string initial values and can misinterpret normal app-lane calls as explicit-id authoring.

Evidence:

docs position normal app/scoped authoring as handle-first:

app_surface_reference.md (line 183)




scoped runtime implementation branches on typeof firstArg === "string" && arguments.length >= 2:

scopes.js (line 201)







What I verified:


scope.input('', { debugName: 'x' })

throws scoped authoring requires a non-empty local id




scope.input('value', { debugName: 'x' })

does not create an input with value 'value'

it creates an explicitly named input with local id value

and uses { debugName: 'x' } as the initial value





That is not just confusing. That is a real contract violation relative to the documented “normal app code” lane.

The package’s own roadmap/spec docs encourage patterns that are unsafe under the current runtime behavior.

Evidence:

controller_scope_and_graph_lifecycle_plan.md (line 1007)

signals.runtime.test.mjs (line 2604)




The pattern they encourage is:


graph.scope("editSession").input(value, { id: "serverItemData" })


That can work for many values, but the current scoped input(...) overload becomes unsafe as soon as the first argument is a string value.