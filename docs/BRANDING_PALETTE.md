# VelaRi Cozy Calm Palette

## Direction

VelaRi should use a **quiet, warm, nature-inspired technical palette**. The colors should feel comfortable in long documentation sessions and distinctive enough to separate VelaRi from the typical neon blue or black-and-red developer-tool aesthetic.

The recommended direction combines a deep blue-green foundation, a soft sage accent, a warm paper background, and a restrained clay highlight.

## Core palette

| Token | Name | HEX | Intended use |
| --- | --- | --- | --- |
| `vela-ink` | Forest Ink | `#24343A` | Primary text, dark logo, dark UI surface |
| `vela-night` | Blue Spruce | `#1E3035` | Dark background, terminal/documentation dark mode |
| `vela-spruce` | Calm Spruce | `#527A78` | Primary brand color, links, active controls |
| `vela-sage` | Soft Sage | `#A8C3B0` | Secondary accent, selected states, supporting shapes |
| `vela-mist` | Quiet Mist | `#DCE9E2` | Pale accent surface, code block tint, light illustrations |
| `vela-paper` | Warm Paper | `#F7F3EA` | Main light background |
| `vela-sand` | Oat Sand | `#E8DDCA` | Cards, borders, subtle background variation |
| `vela-clay` | Muted Clay | `#C9826B` | Small highlights, calls to action, important emphasis |
| `vela-white` | Soft White | `#FFFDF8` | Logo reversal and high-emphasis light surfaces |

## Recommended combinations

| Context | Background | Foreground or accent | Notes |
| --- | --- | --- | --- |
| Light documentation | `vela-paper` | `vela-ink` | Default reading surface |
| Light brand header | `vela-mist` | `vela-night` and `vela-spruce` | Calm and welcoming |
| Dark documentation | `vela-night` | `vela-paper` | Use `vela-sage` for links and highlights |
| Dark logo lockup | `vela-night` | `vela-white` and `vela-sage` | Avoid muted clay for small text |
| Call-to-action | `vela-spruce` | `vela-white` | Use sparingly |
| Important accent | `vela-clay` | `vela-ink` | Reserve for emphasis, not primary navigation |
| Code block | `vela-ink` | `vela-paper` with `vela-sage` accents | Keep syntax colors low-saturation |

## Suggested semantic tokens

```text
--color-bg: #F7F3EA
--color-surface: #FFFDF8
--color-surface-muted: #E8DDCA
--color-text: #24343A
--color-text-muted: #527A78
--color-brand: #527A78
--color-brand-soft: #A8C3B0
--color-link: #386563
--color-focus: #C9826B
--color-dark-bg: #1E3035
--color-dark-surface: #24343A
--color-dark-text: #F7F3EA
```

## Accessibility guidance

Use `vela-ink` for body text on `vela-paper`, `vela-white`, `vela-mist`, or `vela-sand`. Use `vela-white` for text on `vela-spruce` or `vela-night`. The clay color should be treated as an accent and should not carry long body text on a light background. Figma should validate final typography combinations against WCAG contrast targets before the palette is locked.

The logo must also be delivered in pure black, pure white, and one-color `vela-ink` variants so it remains usable in print, terminal-adjacent contexts, and monochrome tooling.

## Visual character

The palette should be applied with generous whitespace, soft but controlled corner radii, subtle borders, and low-noise surfaces. Avoid overusing gradients. If a gradient is explored, use a gentle transition from `vela-mist` through `vela-sage` and provide a flat-color fallback using `vela-spruce`.

## Figma instruction

Use these tokens as the starting palette, not as an unchangeable constraint. Explore small hue adjustments while preserving the same emotional direction: warm paper, blue-green structure, soft sage calm, and muted clay warmth. Keep the final palette to a small number of reusable colors rather than adding many near-duplicates.
