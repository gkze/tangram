use ../../test.nu *
use ../lib/esm.nu

# The compiler follows external declarations and their relative imports.
let server = server spawn
let path = esm fixture
let output = tg check $path | complete
success $output
