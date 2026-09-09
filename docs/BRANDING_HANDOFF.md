# VelaRi Branding Handoff

This brief separates visual design decisions from the compiler roadmap. The compiler and CLI should remain functional without loading image assets.

## Required assets

| Asset | Preferred formats | Minimum variants |
| --- | --- | --- |
| Language mark | SVG, PNG | Light background and dark background |
| Wordmark | SVG | Horizontal and compact layouts |
| GitHub banner | SVG or 3:1 PNG | Center-safe composition for social cropping |
| Language icon | SVG, PNG, ICO | 16, 32, 128, and 256 pixel exports |
| Syntax/documentation icon | SVG, PNG | Monochrome and accent-color variants |

## Recommended filenames

```text
velari-mark.svg
velari-wordmark.svg
velari-banner.svg
velari-icon.svg
velari-icon.ico
velari-syntax.svg
```

## Design constraints

The mark should remain legible at small sizes. The icon should work without the wordmark. The banner should preserve a quiet center area because GitHub and social previews may crop from the edges. Vector files should avoid unnecessary embedded raster images. Text in the logo should be converted to paths or use a clearly documented font.

## Technical handoff checklist

Before adding assets to the repository, provide the source vector files, exported variants, color values, font or licensing information, and a short usage note. Store the final assets under `assets/branding/`. Do not place temporary explorations in the release archive.

## Future integration points

The assets can later be used by the GitHub README, release notes, a documentation website, an editor extension, and a project generator template. None of those integrations should become a dependency of the compiler binary.

## Palette direction

The recommended visual direction is cozy and calm: warm paper, blue-green structure, soft sage accents, and muted clay highlights. See [BRANDING_PALETTE.md](BRANDING_PALETTE.md) for the complete token set, combinations, and accessibility guidance.
