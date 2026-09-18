Build a documentation reader for a workspace product.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Recharts is already installed. Use it for any charts requested by this task.

Use the six articles in `references/data.json`. Copy the data into `src/` before using it. Preserve
the supplied titles, section order, text, and code examples.

Arrange the page as a grouped article sidebar, an article reading area, and a table of contents for
the selected article. This is a reading interface, not a dashboard. The sidebar groups are Getting
started, Guides, and Reference. Each group can collapse or expand independently. Start with all
groups expanded and Workspace overview selected.

The article header shows its group, title, description, and updated date. Render paragraph blocks,
bulleted lists, ordered lists, notes, code examples, and reference tables according to their type.
Use a clear hierarchy for article titles, section headings, and body text. Keep code formatting and
line breaks. Code blocks may scroll horizontally within the article.

Selecting an article updates the reading area and table of contents, marks the selected sidebar
item, and starts at the top of the article. Keep the sidebar group expansion choices. The table of
contents lists the article's section headings. Selecting an entry scrolls to that section. Mark the
section currently at the top of the reading area as active.

Provide a Copy action on each code example. Copy only the code text. Show Copied for two seconds
after success. If copying fails, show a failure message instead. Other code examples keep their own
copy state.

At the bottom, show Previous article and Next article using the supplied article order across all
groups. Omit Previous on the first article and Next on the last. These controls use the same
selection behavior as the sidebar.

Keep all state local. Do not add article editing, search, login, or external navigation. Do not call
a network service or preserve the selected article after a page reload. Do not add dependencies or
change files outside `src/`. In your final response, state what you built and how you checked it.
