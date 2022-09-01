# OTC - Options to Composition

OTC is a tool to convert Vue.js projects written with the Options API, to use the Composition API


TODO:
- Don't treat props as refs
- Support injects
- Support watchers
- Don't tread \$emit as refs (use ctx.\$emit)
- Support functional components


### Storing data on the visitor
The `Visitor` struct needs to store the keys from props, injects, and anything else that is is accessed through `this` in the options API, but will not *not* become a Ref/ComputedRef when converted.
