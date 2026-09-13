use ../../test.nu *
use ../lib/esm.nu

# Changing package conditions invalidates the captured graph.
let server = server spawn
let path = esm fixture
let id = tg checkin --watch $path
'{"conditions":[]}' | save --force ($path | path join 'tangram.packages.json')
let changed = tg checkin --watch $path
assert not equal $id $changed
let output = tg build $changed --cached=false
assert equal ($output | from json) [43 true true legacy]
