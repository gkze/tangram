use ../../test.nu *

# Opt-in acceptance with a provisioned, unmodified Effect installation.
let source = $env.TANGRAM_TEST_EFFECT_PACKAGE? | default ''
if $source == '' {
	skip_test 'set TANGRAM_TEST_EFFECT_PACKAGE to a prepared Effect package directory'
}
let source = $source | path expand
let package = open ($source | path join package.json)
assert equal $package.name effect
# This real package graph exceeds the harness maps sized for tiny fixtures.
let server = server spawn --config {
	index: {kind: lmdb, map_size: 1_073_741_824}
	store: {kind: lmdb, map_size: 1_073_741_824}
}
let path = artifact {
	'tangram.packages.json': '{"conditions":["tangram"]}'
	'package.json': ({type: module, dependencies: {effect: $package.version}} | to json)
	'tangram.ts': ''
}
mkdir ($path | path join node_modules)
# Dereference the provisioner's links without changing any package contents.
^cp -R -L $source ($path | path join node_modules effect)
for entry in [
	{
		source: 'import * as Effect from "effect/Effect"; export default () => Effect.runSync(Effect.map(Effect.succeed(41), n => n + 1));'
		expected: 42
	}
	{
		source: '
			import * as Effect from "effect/Effect";
			import * as Schema from "effect/Schema";
			import * as Encoding from "effect/Encoding";
			export default () => {
				const person = Schema.decodeUnknownSync(Schema.Struct({name: Schema.String, age: Schema.NumberFromString}))({name:"Ada", age:"42"});
				const total = Effect.runSync(Effect.flatMap(Effect.succeed(person.age), n => Effect.succeed(n + 1)));
				return [person.name, total, Encoding.encodeBase64("hé😀")];
			};
		'
		expected: [Ada 43 'aMOp8J+YgA==']
	}
	{
		source: 'import { Effect, Schema } from "effect"; export default () => Effect.runSync(Effect.succeed(Schema.decodeUnknownSync(Schema.NumberFromString)("42")));'
		expected: 42
	}
] {
	$entry.source | save --force ($path | path join tangram.ts)
	assert equal (tg build $path --cached=false | from json) $entry.expected
	let id = tg checkin $path
	mv $path ($path + '.moved')
	assert equal (tg build $id --cached=false | from json) $entry.expected
	mv ($path + '.moved') $path
}
