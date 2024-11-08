use std::io::{self, Write};
use std::fs;
use std::path::PathBuf;
use chrono::Local;

#[derive(Debug)]
enum Command {
    Write(String),
    Search(String),
    List,
    MultiLine,
    Save(Option<String>),
    Load(String),
    ListFiles,
    NewNote(bool),
    Help,          
    Quit,
    Invalid(String),
}

struct Editor {
    content: String,
    modified: bool,
    in_multi_line: bool,
    current_block: String,
    current_file: Option<PathBuf>,
    notes_dir: PathBuf,
}

impl Editor {
    fn new() -> io::Result<Self> {
        let home = dirs::home_dir().expect("Could not find home directory");
        let notes_dir = home.join(".notes");
        fs::create_dir_all(&notes_dir)?;

        Ok(Editor {
            content: String::new(),
            modified: false,
            in_multi_line: false,
            current_block: String::new(),
            current_file: None,
            notes_dir,
        })
    }

    fn format_content(&self) -> String {
        let mut formatted = String::new();
        let lines: Vec<&str> = self.content.lines().collect();
        let mut in_section = false;

        for line in lines {
            if line.starts_with("****") {
                if in_section {
                    formatted.push('\n');
                }
                formatted.push_str(&format!("\n{}\n{}\n", line, "=".repeat(line.len())));
                in_section = true;
            } else if line.starts_with("- ") {
                formatted.push_str(&format!("  {}\n", line));
            } else if line.starts_with("HTTP/") || line.starts_with("GET ") || line.starts_with("POST ") {
                formatted.push_str(&format!("  > {}\n", line));
            } else if line.is_empty() {
                formatted.push('\n');
            } else {
                formatted.push_str(&format!("{}\n", line));
            }
        }

        formatted
    }

    fn save_current(&mut self) -> io::Result<()> {
        if !self.modified {
            println!("No changes to save");
            return Ok(());
        }

        let file_path = if let Some(path) = &self.current_file {
            path.clone()
        } else {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("note_{}.md", timestamp);
            self.notes_dir.join(filename)
        };

        fs::write(&file_path, &self.format_content())?;
        self.current_file = Some(file_path.clone());
        self.modified = false;
        println!("[+] Saved to {}", file_path.file_name().unwrap().to_string_lossy());
        Ok(())
    }

    fn load_file(&mut self, name: &str) -> io::Result<()> {
        let path = if name.ends_with(".md") {
            self.notes_dir.join(name)
        } else {
            self.notes_dir.join(format!("{}.md", name))
        };

        if path.exists() {
            self.content = fs::read_to_string(&path)?;
            self.current_file = Some(path.clone());
            self.modified = false;
            println!("[+] Loaded {}", path.file_name().unwrap().to_string_lossy());
        } else {
            println!("File not found: {}", name);
        }
        Ok(())
    }

    fn list_saved_notes(&self) -> io::Result<()> {
        println!("\nSaved Notes:");
        println!("{}", "=".repeat(40));
        
        let mut notes: Vec<_> = fs::read_dir(&self.notes_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "md"))
            .collect();
        
        if notes.is_empty() {
            println!("No saved notes found.");
            println!("{}", "=".repeat(40));
            return Ok(());
        }

        notes.sort_by(|a, b| b.metadata().unwrap().modified().unwrap()
                            .cmp(&a.metadata().unwrap().modified().unwrap()));

        let max_name_len = notes.iter()
            .map(|entry| entry.path().file_name().unwrap().to_string_lossy().len())
            .max()
            .unwrap_or(0);

        for (idx, entry) in notes.iter().enumerate() {
            let modified = entry.metadata()?.modified()?;
            let modified_time = chrono::DateTime::<Local>::from(modified);
            let filename = entry.path().file_name().unwrap().to_string_lossy().into_owned();
            println!("{:2}. {:<width$} ({})", 
                    idx + 1,
                    filename,
                    modified_time.format("%Y-%m-%d %H:%M"),
                    width = max_name_len);
        }
        println!("{}", "=".repeat(40));
        Ok(())
    }

    // New: Help command implementation
    fn show_help(&self) {
        println!("\nCommands:");
        println!("  :quit              -> exit editor");
        println!("  :list              -> show formatted note");
        println!("  :save [name]       -> save note (with optional name)");
        println!("  :ls                -> list saved notes");
        println!("  :load <name>       -> load note");
        println!("  :search <text>     -> search for text");
        println!("  :ml                -> start/end multi-line input");
        println!("  :n  / :n!          -> new note (with/without warning)");
        println!("  :help              -> show this help");
    }

    fn parse_command(input: &str, in_multi_line: bool) -> Command {
        let input = input.trim();
        
        if in_multi_line {
            if input == ":ml" {
                return Command::MultiLine;
            }
            return Command::Write(input.to_string());
        }

        if input.starts_with(':') {
            let parts: Vec<&str> = input[1..].split_whitespace().collect();
            match parts.get(0).map(|&s| s) {
                Some("h") | Some("help") => Command::Help,  // New: Help command match
                Some("q") | Some("quit") => Command::Quit,
                Some("l") | Some("list") => Command::List,
                Some("ls") | Some("files") => Command::ListFiles,
                Some("ml") => Command::MultiLine,
                Some("n") => Command::NewNote(false),
                Some("n!") => Command::NewNote(true),
                Some("search") => {
                    if parts.len() > 1 {
                        Command::Search(parts[1..].join(" "))
                    } else {
                        Command::Invalid("Search term required".to_string())
                    }
                }
                Some("save") => {
                    if parts.len() > 1 {
                        Command::Save(Some(parts[1..].join("_")))
                    } else {
                        Command::Save(None)
                    }
                }
                Some("load") => {
                    if parts.len() > 1 {
                        Command::Load(parts[1].to_string())
                    } else {
                        Command::Invalid("Filename required".to_string())
                    }
                }
                _ => Command::Invalid(input.to_string()),
            }
        } else {
            Command::Write(input.to_string())
        }
    }

    fn execute_command(&mut self, command: Command) -> io::Result<bool> {
        match command {
            Command::Write(text) => {
                if self.in_multi_line {
                    self.current_block.push_str(&text);
                    self.current_block.push('\n');
                    print!("  ");
                    io::stdout().flush()?;
                } else {
                    if !text.is_empty() {
                        self.content.push_str(&text);
                        self.content.push('\n');
                        self.modified = true;
                        println!("[+] Added");
                    }
                }
                Ok(true)
            }
            Command::MultiLine => {
                if self.in_multi_line {
                    self.in_multi_line = false;
                    self.content.push_str(&self.current_block);
                    self.modified = true;
                    println!("[+] Multi-line input completed ({} lines added)", 
                            self.current_block.lines().count());
                    self.current_block.clear();
                } else {
                    println!("Multi-line mode started:");
                    println!("  Type your content (one line at a time)");
                    println!("  Use :ml again to finish");
                    println!("---");
                    self.in_multi_line = true;
                    self.current_block.clear();
                }
                Ok(true)
            }
            Command::Save(name_opt) => {
                match name_opt {
                    Some(name) => {
                        let file_path = self.notes_dir.join(format!("{}.md", name));
                        fs::write(&file_path, &self.format_content())?;
                        self.current_file = Some(file_path);
                        self.modified = false;
                        println!("[+] Saved as {}.md", name);
                        println!("  Use :list to view formatted content");
                    }
                    None => {
                        self.save_current()?;
                        println!("  Use :list to view formatted content");
                    }
                }
                Ok(true)
            }
            Command::Load(name) => {
                if self.modified {
                    println!("Current note has unsaved changes.");
                    println!("Save first with :save or force load with :n! then :load");
                } else {
                    self.load_file(&name)?;
                    println!("  Use :list to view formatted content");
                }
                Ok(true)
            }
            Command::Search(term) => {
                let mut found = false;
                let mut results = Vec::new();
                
                for (i, line) in self.content.lines().enumerate() {
                    if line.contains(&term) {
                        found = true;
                        results.push((i + 1, line));
                    }
                }

                if found {
                    println!("\nSearch results for '{}':", term);
                    println!("{}", "=".repeat(40));
                    for (line_num, content) in &results {
                        println!("{:>4}: {}", line_num, content);
                    }
                    println!("{}", "=".repeat(40));
                    println!("Found {} matching line(s)\n", results.len());
                } else {
                    println!("No matches found for '{}'\n", term);
                }
                Ok(true)
            }
            Command::List => {
                if self.content.is_empty() {
                    println!("Note is empty. Start typing to add content.");
                } else {
                    println!("\nCurrent note contents:");
                    println!("{}", "=".repeat(20));
                    println!("{}", self.format_content());
                    println!("{}", "=".repeat(20));
                }
                Ok(true)
            }
            Command::ListFiles => {
                self.list_saved_notes()?;
                println!("Type ':load <name>' to load a note");
                println!("Type ':save <name>' to save current note with a specific name");
                Ok(true)
            }
            Command::NewNote(force) => {
                if self.modified && !force {
                    println!("Note has unsaved changes.");
                    println!("Use :n! to start new without saving, or :save first");
                } else {
                    self.content.clear();
                    self.current_file = None;
                    self.modified = false;
                    println!("[+] Started new note. Editor is empty.");
                    println!("  Start typing or use :ml for multi-line input.");
                }
                Ok(true)
            }
            Command::Help => {    // New: Help command handling
                self.show_help();
                Ok(true)
            }
            Command::Quit => {
                if self.modified {
                    self.save_current()?;
                }
                println!("ciao.");
                Ok(false)
            }
            Command::Invalid(cmd) => {
                println!("Invalid command: {}", cmd);
                println!("Type :help for available commands");  // Updated to mention :help
                Ok(true)
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    
    println!("rustynotes: a minimal text editor");
    println!("Commands:");
    println!("  :quit              -> exit editor");
    println!("  :list              -> show formatted note");
    println!("  :save [name]       -> save note (with optional name)");
    println!("  :ls                -> list saved notes");
    println!("  :load <name>       -> load note");
    println!("  :search <text>     -> search for text");
    println!("  :ml                -> start/end multi-line input");
    println!("  :n  / :n!          -> new note (with/without warning)");
    println!("\nNotes saved in: {}", editor.notes_dir.display());
    println!("Type :ls to see saved notes\n");

    loop {
        if editor.in_multi_line {
            print!("  ");
        } else {
            print!(":> ");
        }
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let command = Editor::parse_command(&input, editor.in_multi_line);
        if !editor.execute_command(command)? {
            break;
        }
    }

    Ok(())
}
