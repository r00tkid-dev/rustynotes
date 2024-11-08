```
1. Tagging/Organization for Notes [DONE]
Add a simple tagging system or directory structure within .notes folder. 

2. Search Enhancements
- fuzzy search:
:> search authntication  # Misspelled
Did you mean "authentication"?
Found in 3 notes...

- regex:
:> search --regex "CVE-\d{4}-\d{4,7}"
# Finds all CVE patterns like CVE-2024-12345

- content-based:
:> search --urls        # Find all URLs in notes
:> search --code php    # Find PHP code blocks
:> search --headers     # Find HTTP headers

3. Command History (History of previous inputs)
- store in memroy vs persist between sessions (.rustynotes_history)?
up and down arrow keys

4. Find and Replace
Implement a :find <term> command to search for a term, and a :replace <term> <replacement> to replace all occurrences of that term. It could also allow for case-sensitive or whole-word-only search options.

5. Note templates
:> template list
Available templates:
* bug-bounty
* research
* general notes

:> new --template bug-bounty
[Creates new note with structure:]
# Target: 
## Scope:

- user-defined (~/.notes/templates)?
# Save current note as template
:> save-template bug-report

# List templates
:> templates
User templates:
  - bug-report.md
  - recon.md
Default templates:
  - basic.md
  - meeting.md

# Use template
:> new bug-report

# with flags
:> new bug-report --target example.com
# Automatically fills $target

6. Note stats
:> stats current
Current note:
- Lines: 156
- Words: 423
- Characters: 2,145
- Code blocks: 3
- Last modified: 2 hours ago

:> stats all
All notes:
- Total notes: 23
- Total size: 1.2MB
- Most used tags: bug-bounty (15), web (10)
- Most active day: Thursday

7. Preview 
:> ls
1. target1-notes.md (2024-02-08 14:30)
2. target2-recon.md (2024-02-08 15:45)

:> preview 1
--- target1-notes.md ---
Target: example.com
Scope: *.example.com
Tags: web, recon

[Press any key to exit preview]

- split preview?
:> search rce --preview
Found in 3 notes:

1. target1.md:
   | ...found RCE in the upload endpoint...
   | POC: curl -X POST...

2. target2.md:
   | RCE through deserialization...
   | Impact: Critical...
```
