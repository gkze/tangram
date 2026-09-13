use ../../test.nu *

# Ordinary JavaScript remains a file outside an explicit package scope.
let server = server spawn
let ordinary = artifact {
	'tangram.ts': 'import file from "./ordinary.js"; export default () => file instanceof tg.File;'
	'ordinary.js': 'export default 42;'
}
let output = tg build $ordinary
assert equal ($output | from json) true
