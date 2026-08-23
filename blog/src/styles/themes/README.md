# Themes

`base/` contains the shared layout and Markdown/Milkdown geometry. Do not copy it
when making a theme.

To add a selectable theme, copy `template/` to a descriptive directory, replace
`theme-id` and the token values in `theme.css`, then:

- Import it from `index.css`.
- Add its id and display label to `THEMES` in `src/consts.ts`.

The navigation selector persists a valid choice in browser `localStorage`.
Use `overrides.css` only when a token cannot express the visual change. Keep
spacing, breakpoints, and layout in `base/`.

Milkdown is covered by `base/markdown-content.css`: it uses the same semantic
tokens as server-rendered Markdown, so its editor and published posts stay in
sync automatically.
