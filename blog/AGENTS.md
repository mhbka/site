## Development

When starting the dev server, use background mode:

```
astro dev --background
```

Manage the background server with `astro dev stop`, `astro dev status`, and `astro dev logs`.

## UI components

- Check the shared components in `src/components/ui` if the component you're looking for exists there first. Do not add raw `<a>`, `<button>`, `<input>`, or `<textarea>` elements in application UI when an equivalent UI component exists.
- When introducing a reusable UI primitive (for example, a dialog, list, or form control) that could be used elsewhere, add it to `src/components/ui` rather than keeping it page- or feature-specific. Keep feature-specific composition and behavior outside that directory.
- As a styling cue, always keep user-facing navigation, page-title, and header text lowercase.

## Styling and themes

- Shared layout, spacing, sizing, responsive rules, and Markdown/Milkdown geometry live in `src/styles/base/`. Keep these theme-independent.
- Selectable themes are imported from `src/styles/themes/index.css`; theme tokens and visual overrides live in `src/styles/themes/<theme>/`.
- Start new themes by copying `src/styles/themes/template/`, then register the id and label in `THEMES` in `src/consts.ts`; see `src/styles/themes/README.md` for the supported token contract and activation steps.
- Use theme tokens for colours, fonts, shadows, borders, and radii. Put a selector override in a theme's `overrides.css` only when a token cannot express the visual change.
- Keep Astro-rendered Markdown and Milkdown aligned through `src/styles/base/markdown-content.css`; do not add separate visual systems for the editor and published posts.

## Documentation

Full documentation: https://docs.astro.build

Consult these guides before working on related tasks:

- [Adding pages, dynamic routes, or middleware](https://docs.astro.build/en/guides/routing/)
- [Working with Astro components](https://docs.astro.build/en/basics/astro-components/)
- [Using React, Vue, Svelte, or other framework components](https://docs.astro.build/en/guides/framework-components/)
- [Adding or managing content](https://docs.astro.build/en/guides/content-collections/)
- [Adding styles or using Tailwind](https://docs.astro.build/en/guides/styling/)
- [Supporting multiple languages](https://docs.astro.build/en/guides/internationalization/)
