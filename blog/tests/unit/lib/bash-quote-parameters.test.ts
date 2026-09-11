import assert from 'node:assert/strict';
import test from 'node:test';

import { BASH_QUOTE_PARAMETERS_STORAGE_KEY, loadBashQuoteParameters, saveBashQuoteParameters } from '../../../src/lib/local-storage/bash-quote-parameters.ts';
import { unrestrictedScoreLimits } from '../../../src/lib/models/bash-quotes.ts';

function withLocalStorage(run: (storage: Storage) => void) {
	const entries = new Map<string, string>();
	const storage = {
		get length() { return entries.size; },
		clear: () => entries.clear(),
		getItem: (key: string) => entries.get(key) ?? null,
		key: (index: number) => [...entries.keys()][index] ?? null,
		removeItem: (key: string) => entries.delete(key),
		setItem: (key: string, value: string) => entries.set(key, value),
	} as Storage;
	const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
	Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: storage });
	try {
		run(storage);
	} finally {
		if (descriptor) Object.defineProperty(globalThis, 'localStorage', descriptor);
		else Reflect.deleteProperty(globalThis, 'localStorage');
	}
}

test('persists bash quote parameters', () => {
	withLocalStorage(() => {
		const parameters = unrestrictedScoreLimits();
		parameters.toxicity = { lower: 0.2, upper: 0.8 };
		saveBashQuoteParameters(parameters);

		assert.deepEqual(loadBashQuoteParameters(), parameters);
	});
});

test('seeds first-time visitors with the default parameters', () => {
	withLocalStorage((storage) => {
		const parameters = loadBashQuoteParameters();

		assert.deepEqual(parameters.toxicity, { lower: 0, upper: 0.2 });
		assert.deepEqual(parameters.severe_toxicity, { lower: 0, upper: 0.01 });
		assert.deepEqual(JSON.parse(storage.getItem(BASH_QUOTE_PARAMETERS_STORAGE_KEY)!), parameters);
	});
});

test('uses defaults for malformed saved parameters', () => {
	withLocalStorage((storage) => {
		storage.setItem(BASH_QUOTE_PARAMETERS_STORAGE_KEY, JSON.stringify({
			toxicity: { lower: 0.2, upper: 0.8 },
			threat: { lower: 0.9, upper: 0.1 },
		}));

		const parameters = loadBashQuoteParameters();
		assert.deepEqual(parameters.toxicity, { lower: 0.2, upper: 0.8 });
		assert.deepEqual(parameters.threat, { lower: 0, upper: 1 });
	});
});
