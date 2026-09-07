import { join } from 'node:path';

import { asyncBufferFromFile, parquetReadObjects } from 'hyparquet';
import { SCORE_NAMES, type BashQuote, type ScoreLimits } from './models/bash-quotes.ts';

export { SCORE_NAMES, unrestrictedScoreLimits, type BashQuote, type ScoreLimits } from './models/bash-quotes.ts';

export function matchesScoreLimits(quote: BashQuote, limits: ScoreLimits): boolean {
	return SCORE_NAMES.every((name) => quote[name] >= limits[name].lower && quote[name] <= limits[name].upper);
}

type LoadQuotes = () => Promise<BashQuote[]>;

export class BashQuoteStore {
	private quotesPromise?: Promise<BashQuote[]>;
	private readonly loadQuotes: LoadQuotes;

	constructor(loadQuotes: LoadQuotes = readQuotesFromParquet) {
		this.loadQuotes = loadQuotes;
	}

	async getRandomQuote(limits: ScoreLimits): Promise<BashQuote | null> {
		const quotes = await this.getQuotes();
		let selected: BashQuote | null = null;
		let matchingCount = 0;

		for (const quote of quotes) {
			if (!matchesScoreLimits(quote, limits)) continue;
			matchingCount += 1;
			if (Math.random() < 1 / matchingCount) selected = quote;
		}

		return selected;
	}

	async getQuoteCount(limits: ScoreLimits): Promise<number> {
		const quotes = await this.getQuotes();
		return quotes.filter((quote) => matchesScoreLimits(quote, limits)).length;
	}

	private getQuotes(): Promise<BashQuote[]> {
		this.quotesPromise ??= this.loadQuotes();
		return this.quotesPromise;
	}
}

async function readQuotesFromParquet(): Promise<BashQuote[]> {
	const directory = process.env.NODE_ENV === 'production' ? 'data' : join('src', 'assets');
	const file = await asyncBufferFromFile(join(process.cwd(), directory, 'bash_org_max_5_lines_scored.parquet'));
	const rows = await parquetReadObjects({ file, columns: ['qid', 'score', 'quote', ...SCORE_NAMES] });

	return rows.map((row) => ({
		qid: String(row.qid),
		score: Number(row.score),
		quote: String(row.quote),
		toxicity: Number(row.toxicity),
		severe_toxicity: Number(row.severe_toxicity),
		threat: Number(row.threat),
		insult: Number(row.insult),
		identity_attack: Number(row.identity_attack),
		sexual_explicit: Number(row.sexual_explicit),
	}));
}

export const bashQuoteStore = new BashQuoteStore();
