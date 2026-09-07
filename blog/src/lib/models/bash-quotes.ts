export const SCORE_NAMES = [
	'toxicity',
	'severe_toxicity',
	'threat',
	'insult',
	'identity_attack',
	'sexual_explicit',
] as const;

export type ScoreName = (typeof SCORE_NAMES)[number];

export interface QuoteScores {
	toxicity: number;
	severe_toxicity: number;
	threat: number;
	insult: number;
	identity_attack: number;
	sexual_explicit: number;
}

export interface BashQuote extends QuoteScores {
	qid: string;
	score: number;
	quote: string;
}

export type ScoreLimits = Record<ScoreName, { lower: number; upper: number }>;

export const unrestrictedScoreLimits = (): ScoreLimits =>
	Object.fromEntries(SCORE_NAMES.map((name) => [name, { lower: 0, upper: 1 }])) as ScoreLimits;
