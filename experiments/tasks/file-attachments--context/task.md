Build a compact “Attachments” section using the installed Astryx components.

Add a drop zone with a “Choose files” button. Accept PDF and PNG files up to 5 MB each. Show an
inline error for unsupported or oversized files.

Below it, show file rows with an icon, name, size, status, and Remove button.

Start with two example files:

- Project brief.pdf — 1.2 MB — Uploaded.
- Brand assets.png — 840 KB — Upload failed, with a Retry button.

For accepted files and retries, simulate upload progress over two seconds, then show Uploaded. Do
not send files anywhere.

Allow removing any file. When the list is empty, keep the drop zone visible.

Build only this attachment section. Keep state local. Do not add navigation, file previews, network
calls, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
