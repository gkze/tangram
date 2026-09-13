use ../../test.nu *
use ../lib/esm.nu

# Checked-out package dependencies can be captured and executed again.
let server = server spawn
let path = esm fixture
let id = tg checkin $path
let output = $path + '.checkout'
tg checkout $id --path $output | ignore
mv $path ($path + '.moved')
let second = tg checkin $output
assert equal ((tg build $second --cached=false) | from json) [42 true true legacy]
