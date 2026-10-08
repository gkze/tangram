use ../../lib/test.nu *

# TextEncoder encodes UTF-8, replaces unpaired surrogates, and writes only whole scalar values with encodeInto.

let local = server spawn

let path = artifact {
	tangram.ts: '
		export default function () {
			let encoder = new TextEncoder();
			let bytes = new Uint8Array(8).fill(99);
			return {
				encode: [...encoder.encode("Aé😀\ud800Z")],
				encoding: encoder.encoding,
				full: encoder.encodeInto("😀é", bytes.subarray(1, 4)),
				partial: encoder.encodeInto("😀é", bytes.subarray(1, 6)),
				written: [...bytes],
			};
		}
	'
}

let output = tg build $path | from json
assert equal $output {
	encode: [65 195 169 240 159 152 128 239 191 189 90]
	encoding: "utf-8"
	full: { read: 0, written: 0 }
	partial: { read: 2, written: 4 }
	written: [99 240 159 152 128 99 99 99]
}
