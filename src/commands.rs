#[derive(Debug)]
pub enum Command {
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

pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str, in_multi_line: bool, edit_mode: bool) -> Command {
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
                Some("save") if edit_mode => Command::EditSave,
                Some("cancel") if edit_mode => Command::EditCancel,
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
}