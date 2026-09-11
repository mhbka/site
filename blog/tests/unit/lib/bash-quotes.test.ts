import assert from 'node:assert/strict';
import test from 'node:test';

import { BashQuoteStore, matchesScoreLimits, unrestrictedScoreLimits, type BashQuote } from '../../../src/lib/bash-quotes.ts';

const quote: BashQuote = {
	qid: '#1', score: 1, quote: 'hello', toxicity: 0.2, severe_toxicity: 0.1,
	threat: 0.3, insult: 0.4, identity_attack: 0.5, sexual_explicit: 0.6,
};

test('matches quotes only when every score is within its limits', () => {
	const limits = unrestrictedScoreLimits();
	limits.insult = { lower: 0.4, upper: 0.4 };
	assert.equal(matchesScoreLimits(quote, limits), true);
	limits.threat.upper = 0.2;
	assert.equal(matchesScoreLimits(quote, limits), false);
});

test('loads data once and returns the number of matching quotes', async () => {
	let loads = 0;
	const store = new BashQuoteStore(async () => {
		loads += 1;
		return [quote, { ...quote, qid: '#2', toxicity: 0.9 }];
	});
	const limits = unrestrictedScoreLimits();
	limits.toxicity.upper = 0.5;

	assert.equal(await store.getQuoteCount(limits), 1);
	assert.equal((await store.getRandomQuote(limits))?.qid, '#1');
	assert.equal(loads, 1);
});
