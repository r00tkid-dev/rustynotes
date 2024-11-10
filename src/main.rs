use chrono::{DateTime, Local};
use rustyline::error::ReadlineError;
use rustyline::Editor as LineEditor;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
enum Command {
    Write(String),
    Search(String),
    List,
    MultiLine,
    Edit,
    EditLine(usize),
    EditSave,
    EditCancel,
    Save(Option<String>),
    Load(String),
    ListFiles,
    NewNote(bool),
    Tag(String),
    ListTags,
    ListByTag(String),
    Stats,
    Help,
    Quit,
    Invalid(String),
}

#[derive(Debug)]
struct NoteStats {
    lines: usize,
    words: usize,
    chars: usize,
    size_bytes: u64,
    last_modified: String,
    total_notes: usize,
    total_size: String,
    top_tags: Vec<(String, usize)>,
}

struct Editor {
    content: String,
    modified: bool,
    in_multi_line: bool,
    current_block: String,
    current_file: Option<PathBuf>,
    notes_dir: PathBuf,
    current_tags: Vec<String>,
    stats_cache: Option<NoteStats>,
    stats_dirty: bool,
    edit_mode: bool,
    edit_buffer: Vec<String>,
}

impl Editor {
    fn new() -> io::Result<Self> {
        let home = dirs::home_dir().expect("could not find home directory");
        let notes_dir = home.join(".notes");
        fs::create_dir_all(&notes_dir)?;

        Ok(Editor {
            content: String::new(),
            modified: false,
            in_multi_line: false,
            current_block: String::new(),
            current_file: None,
            notes_dir,
            current_tags: Vec::new(),
            stats_cache: None,
            stats_dirty: true,
            edit_mode: false,
            edit_buffer: Vec::new(),
        })
    }

    fn invalidate_stats_cache(&mut self) {
        self.stats_cache = None;
        self.stats_dirty = true;
    }

    fn load_file(&mut self, name: &str) -> io::Result<()> {
        let path = if name.ends_with(".md") {
            self.notes_dir.join(name)
        } else {
            self.notes_dir.join(format!("{}.md", name))
        };

        if path.exists() {
            let content = fs::read_to_string(&path)?;

            self.current_tags.clear();
            if content.starts_with("---\n") {
                if let Some(end) = content.find("\n---\n") {
                    let metadata = &content[4..end];
                    if let Some(tags) = metadata.strip_prefix("tags: ") {
                        self.current_tags = tags.split(", ").map(|s| s.to_string()).collect();
                        self.content = content[end + 5..].to_string();
                    } else {
                        self.content = content;
                    }
                } else {
                    self.content = content;
                }
            } else {
                self.content = content;
            }

            self.current_file = Some(path.clone());
            self.modified = false;
            self.invalidate_stats_cache();
            println!("[+] loaded {}", path.file_name().unwrap().to_string_lossy());
            if !self.current_tags.is_empty() {
                println!("    tags: {}", self.current_tags.join(", "));
            }
        } else {
            println!("[-] file not found: {}", name);
        }
        Ok(())
    }

    fn calculate_stats(&mut self) -> io::Result<NoteStats> {
        let content = &self.content;
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.chars().count();

        let (size_bytes, last_modified) = if let Some(path) = &self.current_file {
            match fs::metadata(path) {
                Ok(metadata) => (
                    metadata.len(),
                    if let Ok(time) = metadata.modified() {
                        format!("{}", DateTime::<Local>::from(time).format("%Y-%m-%d %H:%M"))
                    } else {
                        "Unknown".to_string()
                    },
                ),
                Err(_) => (content.len() as u64, "not saved yet".to_string()),
            }
        } else {
            (content.len() as u64, "not saved yet".to_string())
        };

        let mut total_size = size_bytes;
        let mut tag_counts = HashMap::new();
        let mut total_notes = 0;

        for tag in &self.current_tags {
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }

        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                if let Some(current_path) = &self.current_file {
                    if entry.path() == *current_path {
                        continue;
                    }
                }

                total_notes += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }

                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.starts_with("---\n") {
                        if let Some(end) = content.find("\n---\n") {
                            let metadata = &content[4..end];
                            if let Some(tags) = metadata.strip_prefix("tags: ") {
                                for tag in tags.split(", ") {
                                    *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !self.content.is_empty() || self.current_file.is_some() {
            total_notes += 1;
        }

        let mut top_tags: Vec<_> = tag_counts.into_iter().collect();
        top_tags.sort_by(|a, b| b.1.cmp(&a.1));
        top_tags.truncate(2);

        let total_size_str = if total_size < 1024 {
            format!("{}B", total_size)
        } else if total_size < 1024 * 1024 {
            format!("{:.1}KB", total_size as f64 / 1024.0)
        } else {
            format!("{:.1}MB", total_size as f64 / (1024.0 * 1024.0))
        };

        let stats = NoteStats {
            lines,
            words,
            chars,
            size_bytes,
            last_modified,
            total_notes,
            total_size: total_size_str,
            top_tags,
        };

        Ok(stats)
    }

    fn update_stats(&mut self) -> io::Result<()> {
        if self.stats_dirty {
            self.stats_cache = Some(self.calculate_stats()?);
            self.stats_dirty = false;
        }
        Ok(())
    }

    fn display_stats(&mut self) -> io::Result<()> {
        self.update_stats()?;
        let stats = self.stats_cache.as_ref().unwrap();

        let mut stats_lines = vec![
            format!(
                "Current Note: {}",
                if let Some(ref path) = self.current_file {
                    path.file_name().unwrap().to_string_lossy()
                } else {
                    "[not saved]".into()
                }
            ),
            format!("lines: {}", stats.lines),
            format!("words: {}", stats.words),
            format!("characters: {}", stats.chars),
            format!(
                "size: {}",
                if stats.size_bytes < 1024 {
                    format!("{}B", stats.size_bytes)
                } else if stats.size_bytes < 1024 * 1024 {
                    format!("{:.1}KB", stats.size_bytes as f64 / 1024.0)
                } else {
                    format!("{:.1}MB", stats.size_bytes as f64 / (1024.0 * 1024.0))
                }
            ),
            format!("all-time notes: {}", stats.total_notes),
            format!("last modified: {}", stats.last_modified),
            format!("total size: {}", stats.total_size),
        ];

        if !stats.top_tags.is_empty() {
            stats_lines.push(format!(
                "most used tags: {}",
                stats
                    .top_tags
                    .iter()
                    .map(|(tag, count)| format!("{} ({})", tag, count))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let banner_lines = vec![
            "    █████   █    ██   ██████ ████████▓██   ██",
            "  ▓██ ▒ ██▒ ██  ▓██▒██    ▒ ▓  ██▒ ▓▒▒██  ██▒",
            "  ▓██ ░▄█ ▒▓██  ▒██░ ▓██▄   ▒ ▓██░ ▒░ ▒██ ██░",
            "  ▒██▀▀█▄  ▓▓█  ░██░ ▒   ██▒░ ▓██▓ ░  ░ ▐██▓░",
            "  ░██▓ ▒██▒▒▒█████▓▒██████▒▒  ▒██▒ ░  ░ ██▒▓░",
            "  ░ ▒▓ ░▒▓░░▒▓▒ ▒ ▒▒ ▒▓▒ ▒ ░  ▒ ░░     ██▒▒▒ ",
            "    ░▒ ░ ▒░░░▒░ ░ ░░ ░▒  ░ ░    ░    ▓██ ░▒░ ",
            "    ░░   ░  ░░░ ░ ░░  ░  ░    ░      ▒ ▒ ░░  ",
            "     ░        ░          ░           ░ ░     ",
            "                                     ░ ░     ",
            "   ███▄    █ ▒█████  ████████▓▓█████   ██████ ",
            "   ██ ▀█   █▒██▒  ██▒▓  ██▒ ▓▒▓█   ▀ ▒██    ▒ ",
            "  ▓██  ▀█ ██▒██░  ██▒▒ ▓██░ ▒░▒███   ░ ▓██▄   ",
            "  ▓██▒  ▐▌██▒██   ██░░ ▓██▓ ░ ▒▓█  ▄   ▒   ██▒",
            "  ▒██░   ▓██░ ████▓▒░  ▒██▒ ░ ░▒████▒▒██████▒▒",
            "  ░ ▒░   ▒ ▒░ ▒░▒░▒░   ▒ ░░   ░░ ▒░ ░▒ ▒▓▒ ▒ ░",
            "  ░ ░░   ░ ▒░ ░ ▒ ▒░     ░     ░ ░  ░░ ░▒  ░ ░",
            "     ░   ░ ░░ ░ ░ ▒    ░         ░   ░  ░  ░  ",
            "           ░    ░ ░              ░  ░      ░  ",
        ];

        let banner_width = 60;
        let padding = 4;

        let max_lines = banner_lines.len().max(stats_lines.len());
        println!();
        for i in 0..max_lines {
            if i < banner_lines.len() {
                print!("{}", banner_lines[i]);
            } else {
                print!("{:banner_width$}", "", banner_width = banner_width);
            }

            print!("{:padding$}", "", padding = padding);

            if i < stats_lines.len() {
                println!("{}", stats_lines[i]);
            } else {
                println!();
            }
        }
        println!();
        Ok(())
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
            } else if line.starts_with("HTTP/")
                || line.starts_with("GET ")
                || line.starts_with("POST ")
            {
                formatted.push_str(&format!("  > {}\n", line));
            } else if line.is_empty() {
                formatted.push('\n');
            } else {
                formatted.push_str(&format!("{}\n", line));
            }
        }

        formatted
    }

    fn add_tag(&mut self, tag: String) {
        let tag = tag.to_lowercase();
        if !self.current_tags.contains(&tag) {
            self.current_tags.push(tag.clone());
            self.modified = true;
            self.invalidate_stats_cache();
            println!("[+] added tag: {}", tag);
        } else {
            println!("[-] tag already exists: {}", tag);
        }
    }

    fn save_current(&mut self) -> io::Result<()> {
        if !self.modified {
            println!("[-] no changes to save");
            return Ok(());
        }

        let file_path = if let Some(path) = &self.current_file {
            path.clone()
        } else {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("note_{}.md", timestamp);
            self.notes_dir.join(filename)
        };

        let mut content = String::new();
        if !self.current_tags.is_empty() {
            content.push_str("---\ntags: ");
            content.push_str(&self.current_tags.join(", "));
            content.push_str("\n---\n");
        }
        content.push_str(&self.format_content());

        fs::write(&file_path, content)?;
        self.current_file = Some(file_path.clone());
        self.modified = false;
        self.invalidate_stats_cache();
        println!(
            "[+] saved to {}",
            file_path.file_name().unwrap().to_string_lossy()
        );
        if !self.current_tags.is_empty() {
            println!("    tags: {}", self.current_tags.join(", "));
        }
        Ok(())
    }
    fn enter_edit_mode(&mut self) {
        self.edit_mode = true;
        self.edit_buffer = self.content.lines().map(String::from).collect();
        println!("\n edit mode commands:");
        println!("  :line N      - edit line N");
        println!("  :save        - save changes");
        println!("  :cancel      - discard changes");
        self.display_numbered_content();
    }

    fn display_numbered_content(&self) {
        println!("\ncurrent content:");
        println!("{}", "=".repeat(40));
        for (i, line) in self.edit_buffer.iter().enumerate() {
            println!("{:4}: {}", i + 1, line);
        }
        println!("{}", "=".repeat(40));
    }

    fn edit_line(&mut self, line_num: usize) -> io::Result<()> {
        if line_num == 0 || line_num > self.edit_buffer.len() {
            println!("[-] invalid line");
            return Ok(());
        }
        let line_idx = line_num - 1;
        let current_line = &self.edit_buffer[line_idx];
        println!("editing line {}:", line_num);
        println!("current: {}", self.edit_buffer[line_idx]);
        println!("new:");
        io::stdout().flush()?;

        let mut line_editor = LineEditor::<(), _>::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        line_editor.set_helper(Some(()));
        match line_editor.readline_with_initial("", (current_line, "")) {
            Ok(new_line) => {
                self.edit_buffer[line_idx] = new_line;
                self.display_numbered_content();
                Ok(())
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n[-] edit cancelled");
                Ok(())
            }
            Err(ReadlineError::Eof) => {
                println!("\n[-] edit cancelled");
                Ok(())
            }
            Err(err) => {
                println!("\nerror: {:?}", err);
                Ok(())
            }

        }

    }

    fn save_edits(&mut self) {
        self.content = self.edit_buffer.join("\n");
        self.modified = true;
        self.edit_mode = false;
        self.invalidate_stats_cache();
        println!("[+] changes saved");
    }

    fn cancel_edits(&mut self) {
        self.edit_mode = false;
        self.edit_buffer.clear();
        println!("[-] changed your mind, huh?");
    }

    fn parse_command(&self, input: &str, in_multi_line: bool) -> Command {
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
                Some("h") | Some("help") => Command::Help,
                Some("q") | Some("quit") => Command::Quit,
                Some("l") | Some("list") => Command::List,
                Some("ls") | Some("files") => Command::ListFiles,
                Some("edit") => Command::Edit,
                Some("line") => {
                    if parts.len() > 1 {
                        if let Ok(num) = parts[1].parse::<usize>() {
                            Command::EditLine(num)
                        } else {
                            Command::Invalid("invalid line number".to_string())
                        }
                    } else {
                        Command::Invalid("line number required..".to_string())
                    }
                }
                Some("save") if self.edit_mode => Command::EditSave,
                Some("cancel") if self.edit_mode => Command::EditCancel,
                Some("ml") => Command::MultiLine,
                Some("n") => Command::NewNote(false),
                Some("n!") => Command::NewNote(true),
                Some("stats") => Command::Stats,
                Some("tag") => {
                    if parts.len() > 1 {
                        Command::Tag(parts[1].to_string())
                    } else {
                        Command::Invalid("tag name required".to_string())
                    }
                }
                Some("tags") => Command::ListTags,
                Some("tagged") => {
                    if parts.len() > 1 {
                        Command::ListByTag(parts[1].to_string())
                    } else {
                        Command::Invalid("tag name required".to_string())
                    }
                }
                Some("search") => {
                    if parts.len() > 1 {
                        Command::Search(parts[1..].join(" "))
                    } else {
                        Command::Invalid("search term required".to_string())
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
                        Command::Invalid("filename required".to_string())
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
                        self.invalidate_stats_cache();
                        println!("[+] added");
                    }
                }
                Ok(true)
            }
            Command::Edit => {
                self.enter_edit_mode();
                Ok(true)
            }
            Command::EditLine(num) => {
                if self.edit_mode {
                    self.edit_line(num)?;
                } else {
                    println!("[-] not in edit mode. use :edit");
                }
                Ok(true)
            }

            Command::EditSave => {
                if self.edit_mode {
                    self.save_edits();
                }
                Ok(true)
            }
            Command::EditCancel => {
                if self.edit_mode {
                    self.cancel_edits();
                }
                Ok(true)
            }

            Command::MultiLine => {
                if self.in_multi_line {
                    self.in_multi_line = false;
                    self.content.push_str(&self.current_block);
                    self.modified = true;
                    self.invalidate_stats_cache();
                    println!(
                        "[+] multi-line input completed ({} lines)",
                        self.current_block.lines().count()
                    );
                    self.current_block.clear();
                } else {
                    println!("multi-line mode started:");
                    println!("  use :ml again to finish");
                    println!("---");
                    self.in_multi_line = true;
                    self.current_block.clear();
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
                    println!("\nsearch results for '{}':", term);
                    println!("{}", "=".repeat(40));
                    for (line_num, content) in &results {
                        println!("{:>4}: {}", line_num, content);
                    }
                    println!("{}", "=".repeat(40));
                    println!("found {} matching line(s)\n", results.len());
                } else {
                    println!("[-] no matches found for '{}'\n", term);
                }
                Ok(true)
            }
            Command::Stats => {
                self.display_stats()?;
                Ok(true)
            }
            Command::List => {
                if self.content.is_empty() {
                    println!("[-] note is empty");
                } else {
                    println!("\ncurrent note:");
                    println!("{}", "=".repeat(40));
                    println!("{}", self.content);
                    if !self.current_tags.is_empty() {
                        println!("tags: {}", self.current_tags.join(", "));
                    }
                    println!("{}", "=".repeat(40));
                }
                Ok(true)
            }
            Command::Save(name_opt) => {
                if let Some(name) = name_opt {
                    let file_path = self.notes_dir.join(format!("{}.md", name));
                    fs::write(&file_path, &self.format_content())?;
                    self.current_file = Some(file_path);
                    self.modified = false;
                    self.invalidate_stats_cache();
                    println!("[+] saved as {}.md", name);
                    println!("  use :list to view formatted content");
                } else {
                    self.save_current()?;
                    println!("  use :list to view formatted content");
                }
                Ok(true)
            }
            Command::Load(name) => {
                if self.modified {
                    println!("[-] current note has unsaved changes.");
                    println!("    save first with :save or force load with :n! then :load");
                } else {
                    self.load_file(&name)?;
                }
                Ok(true)
            }

            Command::NewNote(force) => {
                if self.modified && !force {
                    println!("[-] note has unsaved changes");
                    println!("    use :n! to start new without saving, or :save first");
                } else {
                    self.content.clear();
                    self.current_tags.clear();
                    self.current_file = None;
                    self.modified = false;
                    self.invalidate_stats_cache();
                    println!("[+] started new note");
                }
                Ok(true)
            }
            Command::Tag(tag) => {
                self.add_tag(tag);
                Ok(true)
            }
            Command::ListTags => {
                self.list_tags()?;
                Ok(true)
            }
            Command::ListByTag(tag) => {
                self.list_by_tag(&tag)?;
                Ok(true)
            }
            Command::ListFiles => {
                self.list_saved_notes()?;
                println!("type ':load [name]' to load a note");
                println!("type ':save [name]' to save current note with a specific name");
                Ok(true)
            }
            Command::Help => {
                self.show_help();
                Ok(true)
            }
            Command::Quit => {
                if self.modified {
                    self.save_current()?;
                }
                println!("[+] ciao.");
                Ok(false)
            }
            Command::Invalid(cmd) => {
                println!("[-] invalid command: {}", cmd);
                println!("    Use :help to see available commands");
                Ok(true)
            }
        }
    }

    fn show_help(&self) {
        println!("\nCommands:");
        println!("  :quit               ► exit rustynotes");
        println!("  :n  / :n!           ► new note (with/without warning)");
        println!("  :save [name]        ► save note (with optional name)");
        println!("  :load [name]        ► load note");
        println!("  :ls                 ► list saved notes");
        println!("  :list               ► show current note");
        println!("  :stats              ► show note statistics");
        println!("  :tag [name]         ► add tag to current note");
        println!("    :tags             ► list all tags");
        println!("    :tagged [tag]     ► list notes with specific tag");
        println!("  :search [keyword]   ► search for keyword");
        println!("  :ml                 ► start/end multi-line input");
        println!("  :edit               ► start edit mode");
        println!("    :line N           ► select line to edit");
        println!("    :save             ► save changes");
        println!("    :cancel           ► discard changes");
        println!("                                  ↳ written by r00tkid");
    }

    fn list_saved_notes(&self) -> io::Result<()> {
        println!("\nsaved notes:");
        println!("{}", "=".repeat(40));

        let mut notes: Vec<_> = fs::read_dir(&self.notes_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "md"))
            .collect();

        if notes.is_empty() {
            println!("[-] no saved notes found.");
            println!("{}", "=".repeat(40));
            return Ok(());
        }

        notes.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });

        let max_name_len = notes
            .iter()
            .map(|entry| entry.path().file_name().unwrap().to_string_lossy().len())
            .max()
            .unwrap_or(0);

        for (idx, entry) in notes.iter().enumerate() {
            let modified = entry.metadata()?.modified()?;
            let modified_time = DateTime::<Local>::from(modified);

            let mut tags = Vec::new();
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if content.starts_with("---\n") {
                    if let Some(end) = content.find("\n---\n") {
                        let metadata = &content[4..end];
                        if let Some(tag_list) = metadata.strip_prefix("tags: ") {
                            tags = tag_list.split(", ").map(|s| s.to_string()).collect();
                        }
                    }
                }
            }

            let filename = entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            print!(
                "{:2}. {:<width$} ({})",
                idx + 1,
                filename,
                modified_time.format("%Y-%m-%d %H:%M"),
                width = max_name_len
            );

            if !tags.is_empty() {
                println!(" [{}]", tags.join(", "));
            } else {
                println!();
            }
        }
        println!("{}", "=".repeat(40));
        Ok(())
    }

    fn list_tags(&self) -> io::Result<()> {
        let mut all_tags = HashSet::new();
        let mut tag_counts = HashMap::new();

        for tag in &self.current_tags {
            all_tags.insert(tag.clone());
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }

        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.starts_with("---\n") {
                        if let Some(end) = content.find("\n---\n") {
                            let metadata = &content[4..end];
                            if let Some(tags) = metadata.strip_prefix("tags: ") {
                                tags.split(", ").for_each(|tag| {
                                    all_tags.insert(tag.to_string());
                                    *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
                                });
                            }
                        }
                    }
                }
            }
        }

        if all_tags.is_empty() {
            println!("[-] no tags found");
        } else {
            println!("\navailable tags:");
            println!("{}", "=".repeat(40));
            let mut tags: Vec<_> = all_tags.iter().collect();
            tags.sort();
            for tag in tags {
                println!("  {} ({} notes)", tag, tag_counts.get(tag).unwrap_or(&0));
            }
            println!("{}", "=".repeat(40));
        }
        Ok(())
    }

    fn list_by_tag(&self, tag: &str) -> io::Result<()> {
        let tag = tag.to_lowercase();
        let mut found = false;

        println!("\nnotes tagged with '{}':", tag);
        println!("{}", "=".repeat(40));

        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.starts_with("---\n") {
                        if let Some(end) = content.find("\n---\n") {
                            let metadata = &content[4..end];
                            if let Some(tags) = metadata.strip_prefix("tags: ") {
                                if tags.split(", ").any(|t| t == tag) {
                                    found = true;
                                    println!(
                                        "  {}",
                                        entry.path().file_name().unwrap().to_string_lossy()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found {
            println!("[-] no notes found with tag: {}", tag);
        }
        println!("{}", "=".repeat(40));
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    let mut line_editor = LineEditor::<(), _>::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("rustynotes: a simple cli note-taking tool");
    println!("type :help for commands\n");

    loop {
        let prompt = if editor.in_multi_line { " " } else { ":> " }; // i think this works
        match line_editor.readline(prompt) {
            Ok(line) => {
                let _ = line_editor.add_history_entry(line.as_str());
                let command = editor.parse_command(&line, editor.in_multi_line);
                if !editor.execute_command(command)? {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("ctrl-c");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("ctrl-d");
                break;
            }
            Err(err) => {
                println!("error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
