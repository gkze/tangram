use ../../test.nu *
use ../lib/esm.nu

# V8 preserves a cycle through an imported binding re-export.
let server = server spawn
let path = esm fixture
'import { identity } from "alpha"; export const increment = (n) => identity ? n + 1 : 0;' | save --force ($path | path join node_modules beta index.js)
let id = tg checkin $path
assert equal ((tg build $id) | from json) [42 true true legacy]
