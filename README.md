# rustynotes cheatsheet

## General Commands:
- `:quit`       -> Exit the editor.
- `:list`       -> Show current note in a formatted view.
- `:save [name]` -> Save the current note, optional name.
- `:ls`         -> List all saved notes.
- `:load [name]` -> Load a note by its name.
- `:search [term]` -> Search through all notes for the given term.
- `:ml`         -> Start multi-line input mode, end with `:ml`.

## New Note:
- `:n`          -> Start a new note (with warning if there is unsaved content).
- `:n!`         -> Force start a new note, discarding any unsaved changes.

## Saving Notes:
- `:save name` -> Save the note as a file (with an optional name).

## File Management:
- `:load [name]` -> Load an existing note by its name.
- `:ls`          -> List all saved notes in the directory with timestamps.
- Saved notes are stored in: `/home/user/.notes`.

## Search:
- `:search [term]` -> Search through all saved notes for the specified term.
  - Search results are shown with the note content that contains the term.

## Multi-line Input:
- `:ml`          -> Enter multi-line mode.
  - Write your note and end with `:ml` to stop.
  - Each line entered in multi-line mode gets appended to the current note.

## Miscellaneous:
- Notes are saved in `.md` format by default.
- Timestamps are attached to each saved note (e.g., `note1.md (2024-11-08 01:08)`).

