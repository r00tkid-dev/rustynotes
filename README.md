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
- by r00tkid
```

## Commands:
```
  :help              -> show this help page
  :list              -> show formatted note
  :load [name]       -> load note
  :ls                -> list saved notes
  :ml                -> start/end multi-line input
  :n  / :n!          -> new note (with/without warning)
  :save [name]       -> save note (with optional name)
  :search [keyword]  -> search for keyword
  :tag [name]        -> add tag to current note
  :tags              -> list all tags
  :tagged [tag]      -> list notes with specific tag
  :quit              -> exit editor
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
```
- :load [name]   -> Load an existing note by its name.
- :ls            -> List all saved notes in the directory with timestamps.
```

## Search:
- Search results are shown with the note content that contains the term.
```
- :search [term] -> Search through all saved notes for the specified term.
```
## Tagging:
```
- :tag [name]    -> Adds desired tag to current note
- :tags          -> Lists all tags.
- :tagged [tag]  -> Lists all notes with the specified tag 
```

## Multi-line Input:
- Each line entered in multi-line mode gets appended to the current note.
- Write your note and end with `:ml` to stop.
```
- :ml            -> Enter multi-line mode.
- :ml            -> Close multi-line mode.
```

## Miscellaneous:
- Notes are saved in `.md` format by default, in: `/home/user/.notes`.
- Timestamps are attached to each saved note (e.g., `note1.md (2024-11-08 01:08)`).
