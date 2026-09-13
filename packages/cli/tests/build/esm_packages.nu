use ../../test.nu *
use ../lib/esm.nu

# Captured package imports preserve module identity across bare, subpath, and dynamic imports.
let server = server spawn
let path = esm fixture
let id = tg checkin $path
assert equal ((tg build $id) | from json) [42 true true legacy]
