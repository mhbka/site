## Development

When starting the dev server, use background mode:

```
astro dev --background
```

Manage the background server with `astro dev stop`, `astro dev status`, and `astro dev logs`.

## UI components

- Check the shared components in `src/components/ui` if the component you're looking for exists there first. Do not add raw `<a>`, `<button>`, `<input>`, or `<textarea>` elements in application UI when an equivalent UI component exists.
- When introducing a reusable UI primitive (for example, a dialog, list, or form control) that could be used elsewhere, add it to `src/components/ui` rather than keeping it page- or feature-specific. Keep feature-specific composition and behavior outside that directory.

## Documentation

Full documentation: https://docs.astro.build

Consult these guides before working on related tasks:

- [Adding pages, dynamic routes, or middleware](https://docs.astro.build/en/guides/routing/)
- [Working with Astro components](https://docs.astro.build/en/basics/astro-components/)
- [Using React, Vue, Svelte, or other framework components](https://docs.astro.build/en/guides/framework-components/)
- [Adding or managing content](https://docs.astro.build/en/guides/content-collections/)
- [Adding styles or using Tailwind](https://docs.astro.build/en/guides/styling/)
- [Supporting multiple languages](https://docs.astro.build/en/guides/internationalization/)
