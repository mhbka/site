import type { APIRoute } from 'astro';

import { bashQuoteStore, SCORE_NAMES, unrestrictedScoreLimits, type ScoreLimits } from '../../lib/bash-quotes.ts';

function getScoreLimits(url: URL): ScoreLimits {
	const limits = unrestrictedScoreLimits();

	for (const name of SCORE_NAMES) {
		const lower = url.searchParams.get(`${name}Lower`);
		const upper = url.searchParams.get(`${name}Upper`);
		limits[name] = {
			lower: lower === null ? 0 : parseScore(lower),
			upper: upper === null ? 1 : parseScore(upper),
		};
		if (limits[name].lower > limits[name].upper) throw new RangeError(`${name} lower limit cannot exceed upper limit`);
	}

	return limits;
}

function parseScore(value: string): number {
	const score = Number(value);
	if (!Number.isFinite(score) || score < 0 || score > 1) throw new RangeError('score limits must be between 0 and 1');
	return score;
}

export const GET: APIRoute = async ({ url }) => {
	try {
		const limits = getScoreLimits(url);
		const count = await bashQuoteStore.getQuoteCount(limits);
		const quote = url.searchParams.get('random') === '1' ? await bashQuoteStore.getRandomQuote(limits) : undefined;
		return Response.json({ count, quote });
	} catch (error) {
		const message = error instanceof Error ? error.message : 'unable to load quotes';
		return Response.json({ error: message }, { status: error instanceof RangeError ? 400 : 500 });
	}
};
