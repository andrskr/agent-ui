Build a media attachment manager for a project.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Use `references/data.json` and the three supplied PNG images as the initial attachments. Copy these
files into `src/` before using them. The initial records contain two uploaded images and one failed
image. The supplied sizes match the sample files.

Show a page title, a short description, and a drop area with a Browse images button. Accept multiple
PNG and JPEG images, up to 5 MiB each. Show the allowed types and size limit before selection. Use
the same validation for browsing and dropping. Reject unsupported or oversized files with a message
naming each rejected file. Still accept valid files from the same selection. A later selection
replaces the previous validation messages.

Show attachments as a thumbnail grid. Each item shows its file name, size, and upload status. For
accepted local files, show a preview from the selected file. Assign each item a separate ID; files
with the same name are allowed and remain separate items.

Simulate each new upload over two seconds. Show progress from 0 to 100 percent, then mark it as
uploaded. Each upload progresses independently. New uploads succeed. The initial failed item has a
Retry action and the message Upload failed. Try again. Retry follows the same progress sequence and
succeeds. Prevent repeated retries while it is uploading.

Selecting an uploaded thumbnail opens a preview overlay with a larger image, file name, size, and
Previous and Next controls. Step through uploaded items in grid order without wrapping. Failed or
uploading images cannot open the preview. Closing the preview preserves the grid state.

Uploaded and failed items have selection controls. Uploading items cannot be selected. Show the
selected count, Clear selection, and Remove selected. Removal opens a confirmation dialog naming the
number of items. Cancel preserves selection. Confirm removes those items and clears selection.
Retrying a selected failed item removes it from the selection.

Show uploaded count and total attachment count. Rejected files do not count. When no attachments
remain, show an empty state and keep browsing and dropping available. Release temporary preview URLs
when their items are removed or the component is removed.

Keep all state local. Do not upload files to a service, edit images, or preserve attachments after a
page reload. Do not add dependencies or change files outside `src/`. In your final response, state
what you built and how you checked it.
