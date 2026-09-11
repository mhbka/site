import { join } from 'node:path';

import { asyncBufferFromFile, parquetReadObjects } from 'hyparquet';
import { SCORE_NAMES, type BashQuote, type ScoreLimits } from './models/bash-quotes.ts';

export { SCORE_NAMES, unrestrictedScoreLimits, type BashQuote, type ScoreLimits } from './models/bash-quotes.ts';

/** Reports whether every quote score falls within the chosen limits. */
export function matchesScoreLimits(quote: BashQuote, limits: ScoreLimits): boolean {
	return SCORE_NAMES.every((name) => quote[name] >= limits[name].lower && quote[name] <= limits[name].upper);
}

type LoadQuotes = () => Promise<BashQuote[]>;

/** Lazily loads and selects bash quotes from the scored dataset. */
export class BashQuoteStore {
	private quotesPromise?: Promise<BashQuote[]>;
	private readonly loadQuotes: LoadQuotes;

	constructor(loadQuotes: LoadQuotes = readQuotesFromParquet) {
		this.loadQuotes = loadQuotes;
	}

	/** Selects one matching quote with equal probability. */
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

	/** Counts quotes that match the selected score limits. */
	async getQuoteCount(limits: ScoreLimits): Promise<number> {
		const quotes = await this.getQuotes();
		return quotes.filter((quote) => matchesScoreLimits(quote, limits)).length;
	}

	/** Reuses the in-flight dataset load for every lookup. */
	private getQuotes(): Promise<BashQuote[]> {
		this.quotesPromise ??= this.loadQuotes();
		return this.quotesPromise;
	}
}

/** Reads the scored quote dataset bundled for the current environment. */
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

/** Supplies the shared quote store used by the bash quote page. */
export const bashQuoteStore = new BashQuoteStore();
