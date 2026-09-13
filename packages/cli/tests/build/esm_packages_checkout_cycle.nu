use ../../test.nu *
use ../lib/esm.nu

# Checked-out cyclic package graphs retain their targets and singleton identity.
let server = server spawn
let path = esm fixture
'import { identity } from "alpha"; export const increment = (n) => identity ? n + 1 : 0;' | save --force ($path | path join node_modules beta index.js)
let id = tg checkin $path
let output = $path + '.checkout'
tg checkout $id --path $output | ignore
mv $path ($path + '.moved')
let second = tg checkin $output
assert equal ((tg build $second --cached=false) | from json) [42 true true legacy]
