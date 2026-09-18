Build a workspace command palette with nested project selection.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Recharts is already installed. Use it for any charts requested by this task.

Use the commands, projects, and initial workspace state in `references/data.json`. Copy the data
into `src/` before using it. Do not add commands or projects.

The page behind the palette shows the workspace name, current section, selected project, sidebar
visibility, and an Open commands button. These values must visibly change when their commands run.
The sidebar contains the supplied section names when it is visible. Start with the palette closed.

Open commands opens an overlay with a search field and command results grouped under Navigation,
Projects, and Workspace. Search command labels and keywords without regard to letter case. Ignore
spaces at the start and end. Hide groups that have no matches. Show a clear empty result message
when nothing matches and an action to clear the search.

Navigation commands change the current section and close the palette. Toggle sidebar changes the
sidebar visibility and closes the palette. Its label changes between Hide sidebar and Show sidebar
to describe the next action. A confirmation message names the completed action.

Switch project opens a second level inside the same overlay. Show a breadcrumb, a Back action, and a
project search field. List the three projects and mark the current project. Search names without
regard to letter case. Selecting a project updates the page and closes the palette.

On entering project selection, start its search empty. Back restores the root command search and
results as they were before entering the second level. Closing from either level makes no further
changes. Opening the palette again starts at the root level with an empty search.

Show a Recent section above the normal groups when the root search is empty. It contains up to three
distinct successfully executed commands, newest first. Switching a project counts as one Switch
project command and opens the project step when used again. Reusing a command moves it to the top
without adding a duplicate. Cancelled actions do not enter Recent.

Keep all state local. Do not add routing, remote search, command creation, or a real account menu.
Do not call a network service or preserve changes after a page reload. Do not add dependencies or
change files outside `src/`. In your final response, state what you built and how you checked it.
