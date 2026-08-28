# Resilience

A design that works only with perfect data is not production-ready. Harden the surface against the
inputs, errors, languages, and network conditions that real users bring.

Use this file two ways: to map the states a surface can enter, and to stress a surface before you
ship it. For the words in each state, see `copy.md`. For the visual finish of each state, see
`interface-quality.md`.

## Map the reachable states

List the states the surface can enter. Design each one, or mark it unreachable and give the reason.
A success state alone is not complete.

- Loading: the first load, a pagination load, and a refresh.
- Empty: first use, cleared by the user, no results, an active filter, no permission, and failure.
  Each is a different message and a different next action.
- Sparse: one item, or a few items.
- Populated: the ordinary case.
- Extreme: long text, large values, and many items.
- Validation: a field error and a form error.
- Error: a network failure, a server error, and a timeout.
- Permission: no view, no edit, and read-only.
- In flight: an optimistic update, a rollback, and a conflict.
- Disabled: a control that is not yet usable.
- Destructive: a confirm or an undo path for an action that is hard to reverse.
- Success: the completed outcome.
- Responsive: compact, intermediate, and wide; zoomed; and right-to-left.

## Extreme data

- Test very long text: a name, a title, or a description of 100 or more characters.
- Test very short or absent text: empty, or one character.
- Test large values: thousands, millions, and billions.
- Test many items: 1000 or more rows, or 50 or more options. Use pagination or virtual scrolling. Do
  not load every item at once.
- Test special characters: emoji, accents, right-to-left text, and CJK.
- Let a container expand or truncate on purpose. Set `min-width: 0` on a flex or grid child so it
  can shrink instead of overflow.
- Truncate with intent: one line with an ellipsis, or a line clamp. Never let text spill.

## Localization

- Budget 30 to 40 percent more space for a translation. German often runs longer than English.
- Avoid a fixed width on a text container. Let the layout adapt to the content.
- Format a date, a time, a number, and a currency per locale with the platform `Intl` API. Do not
  hardcode a format.
- Handle plurals with the locale's plural rules. Do not append an "s".
- Support right-to-left with logical properties, such as `margin-inline` and `padding-inline`.
  Mirror a directional icon.
- Use UTF-8 everywhere. Read `astryx docs internationalization`.

## Network and errors

- Handle each response status: 400, show the validation error; 401, send to sign-in; 403, explain
  the permission; 404, show a not-found state; 429, show a rate-limit message; 500, show a generic
  error and a way to get help.
- Show a clear error with a cause and a retry.
- Handle offline, slow, and timed-out connections. Show a skeleton or a progressive load when the
  connection is slow.
- Preserve user input through a validation error. Do not clear the form.
- Contain a failure. Do not block the whole interface when one component fails.

## Concurrency

- Prevent double submission. Disable the control while the request is in flight.
- Cancel a stale request. Handle the race condition.
- Use an optimistic update with a rollback when the request fails.
- Resolve a conflict when two edits collide.

## Permissions

- Distinguish no view, no edit, and read-only.
- Explain why a path is blocked. Name the condition that unblocks it.

## Accessibility under stress

- Make every action reachable by keyboard. Keep a logical tab order. Manage focus inside an overlay.
- Announce a dynamic change with a live region.
- Give a control an accessible name that matches its visible label.
- Test at 200 percent zoom and in high-contrast mode.

## Performance under stress

- Load an image progressively. Reserve its space to prevent layout shift.
- Show a skeleton on a slow connection.
- Debounce a search input. Throttle a scroll handler.
- Clean up on unmount: remove listeners, cancel subscriptions, clear timers, and abort a pending
  request.

## Input validation

- Validate on the client for a fast response: required, format, length, and pattern.
- Validate on the server always. Never trust the client alone. Sanitize every input.
- State the constraint before submission, not after.

## Verify

Stress the surface. Test:

- long text of 100 or more characters, emoji, right-to-left, and CJK;
- large values and 1000 or more items;
- offline, throttled, and timed-out network;
- every response-status error state;
- empty data and every empty variant;
- rapid repeated submission;
- keyboard-only use, a screen reader, 200 percent zoom, and high contrast.

A screenshot of perfect data is not proof.
