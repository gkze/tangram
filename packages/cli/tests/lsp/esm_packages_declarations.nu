use ../../test.nu *
use ../lib/lsp.nu

# Changing an external declaration refreshes diagnostics in the same editor session.
let server = server spawn
let root = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import { value } from "alpha"; const result: number = value; export default () => result;'
	node_modules: {alpha: {
		'package.json': '{"name":"alpha","type":"module","exports":{"types":"./one.d.ts","default":"./index.js"}}'
		'index.js': 'export const value = 1;'
		'one.d.ts': 'export declare const value: number;'
		'two.d.ts': 'export declare const value: string;'
	}}
}
let source = open --raw ($root | path join tangram.ts)
let uri = lsp uri ($root | path join tangram.ts)
mut session = lsp start
$session = lsp send $session (lsp initialize 1)
let initialized = lsp wait_response $session 1
$session = $initialized.session
$session = lsp send_all $session [(lsp initialized) (lsp did_open $uri $source) (lsp diagnostics 2 $uri)]
let first = lsp wait_response $session 2
assert equal $first.message.result.items []
$session = $first.session
'export declare const value: string;' | save --force ($root | path join node_modules alpha one.d.ts)
$session = lsp send $session (lsp diagnostics 3 $uri)
let second = lsp wait_response $session 3
snapshot $second.message.result r#'
	kind: full
	items: range: start: line: 0, character: 37, end: line: 0, character: 43, severity: 1, source: tangram, message: Type 'string' is not assignable to type 'number'.

'#
lsp stop $second.session
