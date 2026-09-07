import { loadBashQuoteParameters } from '../local-storage/bash-quote-parameters.ts';
import { SCORE_NAMES, type BashQuote } from '../models/bash-quotes.ts';

interface BashQuoteResponse {
	count: number;
	quote?: BashQuote | null;
}

async function getBashQuotes(includeQuote: boolean): Promise<BashQuoteResponse> {
	const parameters = new URLSearchParams();
	const limits = loadBashQuoteParameters();

	for (const name of SCORE_NAMES) {
		parameters.set(`${name}Lower`, String(limits[name].lower));
		parameters.set(`${name}Upper`, String(limits[name].upper));
	}
	if (includeQuote) parameters.set('random', '1');

	const response = await fetch(`/api/bash-quotes?${parameters}`);
	const result = await response.json();
	if (!response.ok) throw new Error(result.error);
	return result as BashQuoteResponse;
}

export async function getBashQuoteCount(): Promise<number> {
	return (await getBashQuotes(false)).count;
}

export function getRandomBashQuote(): Promise<BashQuoteResponse> {
	return getBashQuotes(true);
}
