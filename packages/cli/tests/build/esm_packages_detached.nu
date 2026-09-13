use ../../test.nu *
use ../lib/esm.nu

# Captured packages execute after their original source has moved.
let server = server spawn
let path = esm fixture
let id = tg checkin $path
mv $path ($path + '.moved')
assert equal ((tg build $id --cached=false) | from json) [42 true true legacy]
