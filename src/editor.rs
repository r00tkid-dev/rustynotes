use crate::commands::Command;
use crate::file_ops::FileOperations;
use crate::stats::{StatsCalculator, NoteStats};
use rustyline::error::ReadlineError;
use rustyline::Editor as LineEditor;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct Editor {
    pub content: String,
    pub modified: bool,
    pub in_multi_line: bool,
    pub current_block: String,
    pub current_file: Option<PathBuf>,
    pub notes_dir: PathBuf,
    pub current_tags: Vec<String>,
    pub stats_cache: Option<NoteStats>,
    pub stats_dirty: bool,
    pub edit_mode: bool,
    pub edit_buffer: Vec<String>,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        let home = dirs::home_dir().expect("could not find home directory");
        let notes_dir = home.join(".notes");
        std::fs::create_dir_all(&notes_dir)?;

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

    pub fn invalidate_stats_cache(&mut self) {
        self.stats_cache = None;
        self.stats_dirty = true;
    }

    pub fn load_file(&mut self, name: &str) -> io::Result<()> {
        match FileOperations::load_file(&self.notes_dir, name) {
            Ok((content, tags, path)) => {
                self.content = content;
                self.current_tags = tags;
                self.current_file = Some(path.clone());
                self.modified = false;
                self.invalidate_stats_cache();
                println!("[+] loaded {}", path.file_name().unwrap().to_string_lossy());
                if !self.current_tags.is_empty() {
                    println!("    tags: {}", self.current_tags.join(", "));
                }
            }
            Err(e) => {
                println!("[-] {}", e);
            }
        }
        Ok(())
    }

    pub fn update_stats(&mut self) -> io::Result<()> {
        if self.stats_dirty {
            self.stats_cache = Some(StatsCalculator::calculate_note_stats(
                &self.content,
                &self.current_file,
                &self.notes_dir,
                &self.current_tags,
            )?);
            self.stats_dirty = false;
        }
        Ok(())
    }

    pub fn display_stats(&mut self) -> io::Result<()> {
        self.update_stats()?;
        let stats = self.stats_cache.as_ref().unwrap();
        StatsCalculator::display_stats(stats, &self.current_file);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
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

    pub fn save_current(&mut self) -> io::Result<()> {
        if !self.modified {
            println!("[-] no changes to save");
            return Ok(());
        }

        let filename = self.current_file.as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string());

        let file_path = FileOperations::save_file(
            &self.notes_dir,
            &FileOperations::format_content(&self.content),
            &self.current_tags,
            filename.as_deref(),
        )?;

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

    pub fn enter_edit_mode(&mut self) {
        self.edit_mode = true;
        self.edit_buffer = self.content.lines().map(String::from).collect();
        println!("\n edit mode commands:");
        println!("  :line N      - edit line N");
        println!("  :save        - save changes");
        println!("  :cancel      - discard changes");
        self.display_numbered_content();
    }

    pub fn display_numbered_content(&self) {
        println!("\ncurrent content:");
        println!("{}", "=".repeat(40));
        for (i, line) in self.edit_buffer.iter().enumerate() {
            println!("{:4}: {}", i + 1, line);
        }
        println!("{}", "=".repeat(40));
    }

    pub fn edit_line(&mut self, line_num: usize) -> io::Result<()> {
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

    pub fn save_edits(&mut self) {
        self.content = self.edit_buffer.join("\n");
        self.modified = true;
        self.edit_mode = false;
        self.invalidate_stats_cache();
        println!("[+] changes saved");
    }

    pub fn cancel_edits(&mut self) {
        self.edit_mode = false;
        self.edit_buffer.clear();
        println!("[-] changed your mind, huh?");
    }

    pub fn execute_command(&mut self, command: Command) -> io::Result<bool> {
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
                    let file_path = FileOperations::save_file(
                        &self.notes_dir,
                        &FileOperations::format_content(&self.content),
                        &self.current_tags,
                        Some(&name),
                    )?;
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

        let notes = FileOperations::list_saved_notes(&self.notes_dir)?;

        if notes.is_empty() {
            println!("[-] no saved notes found.");
            println!("{}", "=".repeat(40));
            return Ok(());
        }

        let max_name_len = notes
            .iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(0);

        for (idx, (filename, modified_time, tags)) in notes.iter().enumerate() {
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
        let (all_tags, tag_counts) = FileOperations::get_all_tags(&self.notes_dir, &self.current_tags)?;

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
        let found_notes = FileOperations::find_notes_by_tag(&self.notes_dir, tag)?;

        println!("\nnotes tagged with '{}':", tag);
        println!("{}", "=".repeat(40));

        if found_notes.is_empty() {
            println!("[-] no notes found with tag: {}", tag);
        } else {
            for note in found_notes {
                println!("  {}", note);
            }
        }
        println!("{}", "=".repeat(40));
        Ok(())
    }
}