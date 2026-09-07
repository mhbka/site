import { SCORE_NAMES, unrestrictedScoreLimits, type ScoreLimits } from '../models/bash-quotes.ts';

export const BASH_QUOTE_PARAMETERS_STORAGE_KEY = 'bash-quote-picker:parameters:v1';

function isScoreLimit(value: unknown): value is { lower: number; upper: number } {
	if (!value || typeof value !== 'object') return false;
	const { lower, upper } = value as { lower?: unknown; upper?: unknown };
	return typeof lower === 'number'
		&& typeof upper === 'number'
		&& Number.isFinite(lower)
		&& Number.isFinite(upper)
		&& lower >= 0
		&& upper <= 1
		&& lower <= upper;
}

function defaultParameters(): ScoreLimits {
	const defaults = unrestrictedScoreLimits();
	defaults.toxicity = { lower: 0, upper: 0.2 };
	defaults.severe_toxicity = { lower: 0, upper: 0.01 };
	return defaults;
}

function normaliseParameters(value: unknown): ScoreLimits {
	const defaults = defaultParameters();
	if (!value || typeof value !== 'object') return defaults;

	for (const name of SCORE_NAMES) {
		const limit = (value as Record<string, unknown>)[name];
		if (isScoreLimit(limit)) defaults[name] = limit;
	}
	return defaults;
}

export function loadBashQuoteParameters(): ScoreLimits {
	try {
		const stored = localStorage.getItem(BASH_QUOTE_PARAMETERS_STORAGE_KEY);
		if (stored === null) {
			const defaults = defaultParameters();
			saveBashQuoteParameters(defaults);
			return defaults;
		}
		return normaliseParameters(JSON.parse(stored));
	} catch {
		return defaultParameters();
	}
}

export function saveBashQuoteParameters(parameters: ScoreLimits) {
	try {
		localStorage.setItem(BASH_QUOTE_PARAMETERS_STORAGE_KEY, JSON.stringify(normaliseParameters(parameters)));
	} catch {
		// The picker remains usable when storage is blocked or unavailable.
	}
}
