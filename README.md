```
 ________  ___  ___  ________  _________    ___    ___   
|\   __  \|\  \|\  \|\   ____\|\___   ___\ |\  \  /  /|  
\ \  \|\  \ \  \\\  \ \  \___|\|___ \  \_| \ \  \/  / /  
 \ \   _  _\ \  \\\  \ \_____  \   \ \  \   \ \    / /   
  \ \  \\  \\ \  \\\  \|____|\  \   \ \  \   \/  /  /    
   \ \__\\ _\\ \_______\____\_\  \   \ \__\__/  / /      
    \|__|\|__|\|_______|\_________\   \|__|\___/ /       
                       \|_________|       \|___|/        
                                                         
                                                         
 ________   ________  _________  _______   ________      
|\   ___  \|\   __  \|\___   ___\\  ___ \ |\   ____\     
\ \  \\ \  \ \  \|\  \|___ \  \_\ \   __/|\ \  \___|_    
 \ \  \\ \  \ \  \\\  \   \ \  \ \ \  \_|/_\ \_____  \   
  \ \  \\ \  \ \  \\\  \   \ \  \ \ \  \_|\ \|____|\  \  
   \ \__\\ \__\ \_______\   \ \__\ \ \_______\____\_\  \ 
    \|__| \|__|\|_______|    \|__|  \|_______|\_________\
                                             \|_________|
```

## General Commands:
```
- :quit          -> Exit the editor.
- :list          -> Show current note in a formatted view.
- :save [name]   -> Save the current note, optional name.
- :ls            -> List all saved notes.
- :load [name]   -> Load a note by its name.
- :search [term] -> Search through all notes for the given term.
- :ml            -> Start multi-line input mode, end with `:ml`.
```
## New Note:
```
- :n             -> Start a new note (with warning if there is unsaved content).
- :n!            -> Force start a new note, discarding any unsaved changes.
```

## Saving Notes:
```
- :save name     -> Save the note as a file (with an optional name).
```

## File Management:
- Saved notes are stored in: `/home/user/.notes`.
```
- :load [name]   -> Load an existing note by its name.
- :ls            -> List all saved notes in the directory with timestamps.
```

## Search:
- Search results are shown with the note content that contains the term.
```
- :search [term] -> Search through all saved notes for the specified term.
```

## Multi-line Input:
- Each line entered in multi-line mode gets appended to the current note.
- Write your note and end with `:ml` to stop.
```
- :ml            -> Enter multi-line mode.
```

## Miscellaneous:
- Notes are saved in `.md` format by default.
- Timestamps are attached to each saved note (e.g., `note1.md (2024-11-08 01:08)`).
