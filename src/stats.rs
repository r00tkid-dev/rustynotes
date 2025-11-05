use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct NoteStats {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub size_bytes: u64,
    pub last_modified: String,
    pub total_notes: usize,
    pub total_size: String,
    pub top_tags: Vec<(String, usize)>,
}

pub struct StatsCalculator;

impl StatsCalculator {
    pub fn calculate_note_stats(
        content: &str,
        current_file: &Option<PathBuf>,
        notes_dir: &PathBuf,
        current_tags: &[String],
    ) -> std::io::Result<NoteStats> {
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.chars().count();

        let (size_bytes, last_modified) = if let Some(path) = current_file {
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

        for tag in current_tags {
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }

        for entry in fs::read_dir(notes_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                if let Some(current_path) = current_file {
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

        if !content.is_empty() || current_file.is_some() {
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

        Ok(NoteStats {
            lines,
            words,
            chars,
            size_bytes,
            last_modified,
            total_notes,
            total_size: total_size_str,
            top_tags,
        })
    }

    pub fn display_stats(stats: &NoteStats, current_file: &Option<PathBuf>) {
        let mut stats_lines = vec![
            format!(
                "Current Note: {}",
                if let Some(ref path) = current_file {
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
    }
}