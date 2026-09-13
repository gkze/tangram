use ../../test.nu *

# Newly introduced package metadata invalidates resolutions in an ignored package directory.
let server = server spawn
let path = artifact {
	'.gitignore': 'node_modules'
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import { value } from "alpha"; export default () => value;'
	node_modules: {
		alpha: {
			'package.json': '{"name":"alpha","type":"module","exports":"./lib/index.js","imports":{"#value":"./one.js"}}'
			'one.js': 'export const value = 1;'
			lib: {
				'index.js': 'export { value } from "#value";'
				'two.js': 'export const value = 2;'
			}
		}
	}
}
let first = tg checkin --watch $path
assert equal ((tg build $first) | from json) 1
'{"type":"module","imports":{"#value":"./two.js"}}' | save ($path | path join node_modules alpha lib package.json)
let second = tg checkin --watch $path
assert not equal $first $second
assert equal ((tg build $second) | from json) 2
assert equal ((tg build $first --cached=false) | from json) 1
