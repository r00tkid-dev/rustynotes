use chrono::{DateTime, Local};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub struct FileOperations;

impl FileOperations {
    pub fn load_file(
        notes_dir: &PathBuf,
        name: &str,
    ) -> std::io::Result<(String, Vec<String>, PathBuf)> {
        let path = if name.ends_with(".md") {
            notes_dir.join(name)
        } else {
            notes_dir.join(format!("{}.md", name))
        };

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut tags = Vec::new();

            let processed_content = if content.starts_with("---\n") {
                if let Some(end) = content.find("\n---\n") {
                    let metadata = &content[4..end];
                    if let Some(tags_str) = metadata.strip_prefix("tags: ") {
                        tags = tags_str.split(", ").map(|s| s.to_string()).collect();
                        content[end + 5..].to_string()
                    } else {
                        content
                    }
                } else {
                    content
                }
            } else {
                content
            };

            Ok((processed_content, tags, path))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("file not found: {}", name),
            ))
        }
    }

    pub fn save_file(
        notes_dir: &PathBuf,
        content: &str,
        tags: &[String],
        filename: Option<&str>,
    ) -> std::io::Result<PathBuf> {
        let file_path = if let Some(name) = filename {
            notes_dir.join(format!("{}.md", name))
        } else {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            notes_dir.join(format!("note_{}.md", timestamp))
        };

        let mut final_content = String::new();
        if !tags.is_empty() {
            final_content.push_str("---\ntags: ");
            final_content.push_str(&tags.join(", "));
            final_content.push_str("\n---\n");
        }
        final_content.push_str(content);

        fs::write(&file_path, final_content)?;
        Ok(file_path)
    }

    pub fn list_saved_notes(notes_dir: &PathBuf) -> std::io::Result<Vec<(String, DateTime<Local>, Vec<String>)>> {
        let mut notes = Vec::new();

        for entry in fs::read_dir(notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
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

                notes.push((filename, modified_time, tags));
            }
        }

        notes.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(notes)
    }

    pub fn get_all_tags(notes_dir: &PathBuf, current_tags: &[String]) -> std::io::Result<(HashSet<String>, HashMap<String, usize>)> {
        let mut all_tags = HashSet::new();
        let mut tag_counts = HashMap::new();

        for tag in current_tags {
            all_tags.insert(tag.clone());
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }

        for entry in fs::read_dir(notes_dir)? {
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

        Ok((all_tags, tag_counts))
    }

    pub fn find_notes_by_tag(notes_dir: &PathBuf, tag: &str) -> std::io::Result<Vec<String>> {
        let tag = tag.to_lowercase();
        let mut found_notes = Vec::new();

        for entry in fs::read_dir(notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.starts_with("---\n") {
                        if let Some(end) = content.find("\n---\n") {
                            let metadata = &content[4..end];
                            if let Some(tags) = metadata.strip_prefix("tags: ") {
                                if tags.split(", ").any(|t| t == tag) {
                                    found_notes.push(
                                        entry.path().file_name().unwrap().to_string_lossy().into_owned()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(found_notes)
    }

    pub fn format_content(content: &str) -> String {
        let mut formatted = String::new();
        let lines: Vec<&str> = content.lines().collect();
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
}