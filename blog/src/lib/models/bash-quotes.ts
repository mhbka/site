/** Lists the classifier scores available for each bash quote. */
export const SCORE_NAMES = [
	'toxicity',
	'severe_toxicity',
	'threat',
	'insult',
	'identity_attack',
	'sexual_explicit',
] as const;

export type ScoreName = (typeof SCORE_NAMES)[number];

/** Holds the classifier scores attached to a bash quote. */
export interface QuoteScores {
	toxicity: number;
	severe_toxicity: number;
	threat: number;
	insult: number;
	identity_attack: number;
	sexual_explicit: number;
}

/** Represents a scored quote from the bash.org dataset. */
export interface BashQuote extends QuoteScores {
	qid: string;
	score: number;
	quote: string;
}

export type ScoreLimits = Record<ScoreName, { lower: number; upper: number }>;

/** Creates score limits that allow every quote. */
export const unrestrictedScoreLimits = (): ScoreLimits =>
	Object.fromEntries(SCORE_NAMES.map((name) => [name, { lower: 0, upper: 1 }])) as ScoreLimits;
