# OTC - Options to Composition

OTC is a tool to convert Vue.js projects written with the Options API, to use the Composition API


TODO:
- Determine when to use reactive vs ref
- Handle asyncComputed with `ref` + `watchEffect`
- Add `<script setup>` support
- Support functional components
- Handle variable shadowing?


### Storing data on the visitor
The `Visitor` struct needs to store the keys from props, injects, and anything else that is is accessed through `this` in the options API, but will not *not* become a Ref/ComputedRef when converted.
