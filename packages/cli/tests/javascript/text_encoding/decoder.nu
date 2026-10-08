use ../../lib/test.nu *

# TextDecoder decodes buffer sources, replaces malformed sequences, removes one leading byte order mark, and preserves state across streaming calls.

let local = server spawn

let path = artifact {
	tangram.ts: '
		export default function () {
			let decoder = new TextDecoder(" UTF8\t");
			let bytes = (...values: Array<number>) => new Uint8Array(values);
			return {
				encoding: decoder.encoding,
				fatal: decoder.fatal,
				ignoreBOM: decoder.ignoreBOM,
				malformed: decoder.decode(bytes(0xe2, 0x28, 0xa1, 0xed, 0xa0, 0x80)),
				stream: [
					decoder.decode(bytes(0xef), { stream: true }),
					decoder.decode(bytes(0xbb, 0xbf, 0x41, 0xf0, 0x9f), { stream: true }),
					decoder.decode(bytes(0x98, 0x80)),
				],
				truncated: [decoder.decode(bytes(0xe2), { stream: true }), decoder.decode()],
				view: decoder.decode(new DataView(bytes(0, 0x41, 0x42, 0).buffer, 1, 2)),
			};
		}
	'
}

let output = tg build $path | from json
assert equal $output {
	encoding: "utf-8"
	fatal: false
	ignoreBOM: false
	malformed: "�(����"
	stream: ["" "A" "😀"]
	truncated: ["" "�"]
	view: "AB"
}
