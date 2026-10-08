// This module implements the UTF-8 subset of the WHATWG Encoding Standard's TextEncoder and TextDecoder interfaces. UTF-8 is the only encoding that TextEncoder supports, and TextDecoder rejects every other label with a RangeError, which is the standard's behavior for an unsupported label.

// The labels that the WHATWG Encoding Standard assigns to UTF-8.
let labels = new Set([
	"unicode-1-1-utf-8",
	"unicode11utf8",
	"unicode20utf8",
	"utf-8",
	"utf8",
	"x-unicode20utf8",
]);

// The number of code units passed to each String.fromCharCode call, which bounds the argument count.
let chunkSize = 0x2000;

// The results of a decoder step that are not code points.
let continued = -1;
let error = -2;
let prepended = -3;

export class TextEncoder implements globalThis.TextEncoder {
	// Reading this field from an object that is not a TextEncoder throws a TypeError, which is the interface's brand check.
	#brand = true;

	get encoding(): string {
		void this.#brand;
		return "utf-8";
	}

	encode(input = ""): Uint8Array<ArrayBuffer> {
		void this.#brand;
		let source = `${input}`;
		let bytes = new Uint8Array(utf8Length(source));
		encodeInto(source, bytes);
		return bytes;
	}

	encodeInto(
		source: string,
		destination: Uint8Array,
	): TextEncoderEncodeIntoResult {
		void this.#brand;
		if (arguments.length < 2) {
			throw new TypeError("encodeInto requires a source and a destination");
		}
		source = `${source}`;
		if (!(destination instanceof Uint8Array)) {
			throw new TypeError("the destination must be a Uint8Array");
		}
		return encodeInto(source, destination);
	}

	get [Symbol.toStringTag](): string {
		return "TextEncoder";
	}
}

export class TextDecoder implements globalThis.TextDecoder {
	#bomSeen = false;
	#bytesNeeded = 0;
	#bytesSeen = 0;
	#codePoint = 0;
	#doNotFlush = false;
	#fatal: boolean;
	#ignoreBOM: boolean;
	#lowerBoundary = 0x80;
	#queue = new Uint8Array();
	#upperBoundary = 0xbf;

	constructor(label = "utf-8", options: TextDecoderOptions = {}) {
		// Match the label after trimming ASCII whitespace and lowercasing ASCII letters only.
		let normalized = `${label}`
			.replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, "")
			.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
		if (!labels.has(normalized)) {
			throw new RangeError(`the encoding label "${label}" is not supported`);
		}
		let dictionary = toDictionary(options);
		this.#fatal = Boolean(dictionary?.fatal);
		this.#ignoreBOM = Boolean(dictionary?.ignoreBOM);
	}

	get encoding(): string {
		void this.#fatal;
		return "utf-8";
	}

	get fatal(): boolean {
		return this.#fatal;
	}

	get ignoreBOM(): boolean {
		return this.#ignoreBOM;
	}

	decode(
		input?: ArrayBufferLike | ArrayBufferView,
		options: TextDecodeOptions = {},
	): string {
		// Convert the arguments.
		void this.#fatal;
		let bytes = toBytes(input);
		let dictionary = toDictionary(options);

		// Reset the decoder unless the previous call was streaming, then append the input to the bytes that remain queued.
		if (!this.#doNotFlush) {
			this.#reset();
			this.#bomSeen = false;
			this.#queue = new Uint8Array();
		}
		if (this.#queue.length > 0) {
			let queue = new Uint8Array(this.#queue.length + bytes.length);
			queue.set(this.#queue);
			queue.set(bytes, this.#queue.length);
			bytes = queue;
			this.#queue = new Uint8Array();
		}
		this.#doNotFlush = Boolean(dictionary?.stream);

		// Decode the bytes. Each byte produces at most one code unit, and the end of the queue produces at most one more.
		let output = new Uint16Array(bytes.length + 1);
		let length = 0;
		let index = 0;
		while (index < bytes.length) {
			let byte = bytes[index]!;
			if (byte < 0x80 && this.#bytesNeeded === 0) {
				output[length++] = byte;
				index += 1;
				continue;
			}
			let result = this.#step(byte);
			if (result !== prepended) {
				index += 1;
			}
			if (result === error || result === prepended) {
				if (this.#fatal) {
					// The standard leaves the unprocessed bytes in the queue, so a streaming decoder resumes with them.
					this.#queue = bytes.slice(index);
					throw new TypeError("the input is not valid UTF-8");
				}
				output[length++] = 0xfffd;
			} else if (result >= 0x10000) {
				output[length++] = 0xd800 + ((result - 0x10000) >> 10);
				output[length++] = 0xdc00 + ((result - 0x10000) & 0x3ff);
			} else if (result !== continued) {
				output[length++] = result;
			}
		}

		// Handle the end of the queue unless the call is streaming.
		if (!this.#doNotFlush && this.#bytesNeeded !== 0) {
			this.#reset();
			if (this.#fatal) {
				throw new TypeError("the input is not valid UTF-8");
			}
			output[length++] = 0xfffd;
		}

		// Remove a leading byte order mark once per stream unless it is ignored.
		let start = 0;
		if (!this.#ignoreBOM && !this.#bomSeen && length > 0) {
			this.#bomSeen = true;
			if (output[0] === 0xfeff) {
				start = 1;
			}
		}

		// Serialize the code units.
		let string = "";
		for (let offset = start; offset < length; offset += chunkSize) {
			let chunk = output.subarray(offset, Math.min(offset + chunkSize, length));
			string += String.fromCharCode.apply(
				null,
				chunk as unknown as Array<number>,
			);
		}

		return string;
	}

	get [Symbol.toStringTag](): string {
		return "TextDecoder";
	}

	#reset(): void {
		this.#bytesNeeded = 0;
		this.#bytesSeen = 0;
		this.#codePoint = 0;
		this.#lowerBoundary = 0x80;
		this.#upperBoundary = 0xbf;
	}

	// Run the standard's UTF-8 decoder handler on one byte.
	#step(byte: number): number {
		if (this.#bytesNeeded === 0) {
			if (byte <= 0x7f) {
				return byte;
			} else if (byte >= 0xc2 && byte <= 0xdf) {
				this.#bytesNeeded = 1;
				this.#codePoint = byte & 0x1f;
			} else if (byte >= 0xe0 && byte <= 0xef) {
				if (byte === 0xe0) {
					this.#lowerBoundary = 0xa0;
				} else if (byte === 0xed) {
					this.#upperBoundary = 0x9f;
				}
				this.#bytesNeeded = 2;
				this.#codePoint = byte & 0xf;
			} else if (byte >= 0xf0 && byte <= 0xf4) {
				if (byte === 0xf0) {
					this.#lowerBoundary = 0x90;
				} else if (byte === 0xf4) {
					this.#upperBoundary = 0x8f;
				}
				this.#bytesNeeded = 3;
				this.#codePoint = byte & 0x7;
			} else {
				return error;
			}
			return continued;
		}

		// A byte outside the boundaries ends the sequence and is processed again.
		if (byte < this.#lowerBoundary || byte > this.#upperBoundary) {
			this.#reset();
			return prepended;
		}
		this.#lowerBoundary = 0x80;
		this.#upperBoundary = 0xbf;
		this.#codePoint = (this.#codePoint << 6) | (byte & 0x3f);
		this.#bytesSeen += 1;
		if (this.#bytesSeen !== this.#bytesNeeded) {
			return continued;
		}
		let codePoint = this.#codePoint;
		this.#reset();

		return codePoint;
	}
}

// Encode as many whole scalar values as fit, replacing unpaired surrogates with U+FFFD.
let encodeInto = (
	source: string,
	destination: Uint8Array,
): TextEncoderEncodeIntoResult => {
	let read = 0;
	let written = 0;
	while (read < source.length) {
		let codePoint = source.codePointAt(read)!;
		if (codePoint >= 0xd800 && codePoint <= 0xdfff) {
			codePoint = 0xfffd;
		}
		let size = utf8Size(codePoint);
		if (written + size > destination.length) {
			break;
		}
		if (size === 1) {
			destination[written] = codePoint;
		} else if (size === 2) {
			destination[written] = 0xc0 | (codePoint >> 6);
			destination[written + 1] = 0x80 | (codePoint & 0x3f);
		} else if (size === 3) {
			destination[written] = 0xe0 | (codePoint >> 12);
			destination[written + 1] = 0x80 | ((codePoint >> 6) & 0x3f);
			destination[written + 2] = 0x80 | (codePoint & 0x3f);
		} else {
			destination[written] = 0xf0 | (codePoint >> 18);
			destination[written + 1] = 0x80 | ((codePoint >> 12) & 0x3f);
			destination[written + 2] = 0x80 | ((codePoint >> 6) & 0x3f);
			destination[written + 3] = 0x80 | (codePoint & 0x3f);
		}
		read += codePoint > 0xffff ? 2 : 1;
		written += size;
	}
	return { read, written };
};

// Convert an optional buffer source to a view of its bytes.
let toBytes = (input: unknown): Uint8Array => {
	if (input === undefined) {
		return new Uint8Array();
	}
	let view = ArrayBuffer.isView(input);
	let buffer =
		input instanceof ArrayBuffer ||
		(typeof SharedArrayBuffer === "function" &&
			input instanceof SharedArrayBuffer);
	if (!view && !buffer) {
		throw new TypeError(
			"the input must be an ArrayBuffer or an ArrayBufferView",
		);
	}
	let source = input as ArrayBufferLike | ArrayBufferView;

	// A detached buffer, and every view of one, has a byte length of zero and contributes no bytes.
	if (source.byteLength === 0) {
		return new Uint8Array();
	}

	return ArrayBuffer.isView(source)
		? new Uint8Array(source.buffer, source.byteOffset, source.byteLength)
		: new Uint8Array(source);
};

// Apply the Web IDL dictionary conversion, which accepts undefined, null, and objects.
let toDictionary = <T extends object>(value: T | null | undefined) => {
	if (value === undefined || value === null) {
		return undefined;
	}
	if (typeof value !== "object" && typeof value !== "function") {
		throw new TypeError("the options must be an object");
	}
	return value;
};

// Compute the UTF-8 length of a string after unpaired surrogates are replaced with U+FFFD.
let utf8Length = (source: string): number => {
	let length = 0;
	for (let index = 0; index < source.length; index++) {
		let codePoint = source.codePointAt(index)!;
		length += utf8Size(codePoint);
		if (codePoint > 0xffff) {
			index += 1;
		}
	}
	return length;
};

// Compute the number of UTF-8 bytes for a code point, where a surrogate is replaced with U+FFFD, which has three bytes.
let utf8Size = (codePoint: number): number => {
	if (codePoint <= 0x7f) {
		return 1;
	} else if (codePoint <= 0x7ff) {
		return 2;
	} else if (codePoint <= 0xffff) {
		return 3;
	}
	return 4;
};
