# Interface quality

The quality bar for a rendered surface. Load this when you build a surface, make a material visual
change, or review one.

Code that runs is not the bar. Inspect the rendered result, not the source. A screenshot of the
happy path is not proof. This file covers the visual and interaction craft. For the words in each
state, see `copy.md`. For the tokens and components, read `astryx docs`.

## The bar

- Hierarchy. One primary element leads. A few secondary elements support. The rest stays quiet.
- Restraint. Earn every element. Remove what carries no meaning.
- Finish. Design every reachable state, not the success case alone.
- Consistency. Same role, same treatment. Same action, same pattern.
- Low load. Keep the effort on the task, not on the interface.
- Verify the real surface. Judge the rendered result across states, sizes, and input methods.

Product surfaces prioritize stability and scanning. An expressive marketing surface can earn more
expression, but that is not the target here.

## Hierarchy and layout

- Run the squint test. Blur the detail. You should still see the primary element, the secondary
  element, and the major groups, in order.
- Set the spatial thesis first. Name the primary path, what belongs together, and what leads.
- Group by meaning. Use proximity before a container, a border, or a background.
- Build rhythm from the contrast between tight and generous spacing. Do not use one spacing value
  everywhere; that makes everything weigh the same.
- Use the design-system spacing scale, not one-off values. Read `astryx docs spacing`.
- Match density to use frequency and decision complexity.
- Let hierarchy follow product priority, not the framework default.
- Use depth to show state or layering, not decoration. A zero-offset colored halo is decoration; a
  real shadow carries an offset and a soft blur. Read `astryx docs elevation`.
- Keep the DOM and focus order in step with the visual order across viewports.

## Typography

- Give each role a recognizable job: heading, body, label, metadata, and data. A reader tells them
  apart at a glance, without reading the words.
- Do not place two sizes or weights so close that they cannot carry different jobs.
- Keep body copy in a 45–75 character measure.
- Take the body size from the design system's type scale. Go below it only for a dense role or a
  user setting.
- Tune line height to the face, the width, and the language, not a fixed ratio. A wider measure
  needs more leading.
- On a dark surface, add a little line height, a little tracking, and one weight step for light
  text.
- Keep a repeated role identical across surfaces and states.
- Use the fewest families and roles that make the hierarchy clear. Read `astryx docs typography`.
- Test long headings, localization expansion, zoom, missing weights, and font fallback.

## Color

- Build color as roles, not a bag of swatches: surface, text, action, focus, border, status, and
  data. Read `astryx docs color`.
- Let the strongest color own a region or a role. Do not scatter tiny accents.
- Keep the primary action easy to find. Do not spend its color on decoration.
- On a colored surface, tint secondary text from that hue. Never wash it to a generic gray.
- Keep semantic meaning stable across themes. Compose dark mode; do not invert the light theme.
- Meet WCAG AA contrast: body text 4.5:1, large text 3:1, and controls, icons, and focus rings 3:1.
  Check every state, every overlay, and text on an image.
- Never carry meaning by color alone. Add text, a shape, an icon, or position.

## States and interaction

- Design every reachable state: default, hover, focus, active, disabled, loading, empty, error, and
  success.
- Give every control a visible focus ring, a hover state, and a disabled state.
- Show real loading progress, or none. Never invent progress. Keep the control label stable.
- Design each empty-state variant so the user can tell them apart. The variants are mapped in
  `resilience.md`.
- Keep the tab order logical and the focus visible.
- Keep touch targets usable even when the visible mark is small.

## Motion

- Author one moment, not scattered effects. Do not repeat one identical entrance on every section.
- Ease out from an already-visible default. Read `astryx docs motion`.
- Do not limit motion to transform and opacity. Also use blur, clip-path, mask, and shadow when the
  effect stays smooth.
- Keep motion interruptible and cheap. Do not animate only to make the polish visible.
- Respect reduced motion. Stop a nonessential loop when it is hidden.

## Cognitive load

- Working memory holds about four items. At a decision point, keep the visible options to four or
  fewer. Group the rest.
- Show one primary action, one or two secondary actions, and the rest in a menu.
- Chunk information into groups of four or fewer.
- Do not force the user to hold a value from a previous screen. Keep it visible, or repeat it where
  it is needed.
- Sequence the steps. Let the user do one thing at a time.
- Co-locate the information a decision needs. Cut the back-and-forth.

## Browser surfaces

The parts you did not draw still carry the design. Theme them from the palette: text selection, the
caret, custom scrollbars, focus rings, underline offset, and the numerals in tabular data. This is
the cheapest signal that a page was built, not assembled, and the one models skip most often.

## Delight, when earned

- Delight is product character, not generic whimsy. State one thesis: what the user should feel, and
  why that feeling belongs to this product.
- Concentrate it at earned moments: first use, completion, recovery, and mastery.
- Make celebration proportional to frequency and consequence. A routine save should feel certain,
  not celebrated.
- Delight never delays the task, overrides accessibility, or plays sound without consent.
- Generic whimsy is worse than neutral clarity.

## Refuse

These are the category defaults: the tells of work that was assembled, not built. Reach for one only
when the brief earns it.

- Same-size cards of icon, heading, and text as the page structure. A card is the lazy container.
  Nested cards are wrong.
- The hero-metric template: a big number, a small label, supporting stats, and an accent.
- A kicker or eyebrow above a heading. Question it. The heading usually carries its own weight.
- Section numbers (01 / 02 / 03) unless the sequence itself carries information the reader needs.
- A modal for a task that needs neither interruption nor protected focus.
- Gradient text. Emphasis comes from weight or size.
- Glass and blur as decoration rather than a specific effect.
- A colored left or right border above 1px on a card, a list item, a callout, or an alert.
- Sparklines, progress rings, and soft-shadowed rounded rectangles standing in for content.
- Monospace as a costume for "technical", rather than for code, data, or measurement.
- Unicode glyphs or emoji standing in for an icon system. Draw icons in one consistent stroke and
  weight.
- A light or dark theme picked from the product category. Pick the theme from the use scene: who,
  where, and under what light.

## Verify

Inspect the rendered surface, not the source. Test:

- the squint test still shows the primary, the secondary, and the groups, in order;
- every reachable state you changed: default, hover, focus, active, disabled, loading, empty, error,
  and success;
- the compact, intermediate, and wide viewports;
- keyboard order, visible focus, and screen-reader names that match the visual order;
- long content, large values, empty data, localization expansion, and 200% zoom;
- contrast in every state, every overlay, and text on an image;
- no console errors, no layout shift, and no dropped frames;
- browser surfaces themed from the palette.

An automated check is a floor, not proof. Judge the rendered result.
