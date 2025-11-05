mod commands;
mod editor;
mod file_ops;
mod stats;

use commands::CommandParser;
use editor::Editor;
use rustyline::error::ReadlineError;
use rustyline::Editor as LineEditor;
use std::io;

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    let mut line_editor = LineEditor::<(), _>::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("rustynotes: a simple cli note-taking tool");
    println!("type :help for commands\n");

    loop {
        let prompt = if editor.in_multi_line { " " } else { ":> " };
        match line_editor.readline(prompt) {
            Ok(line) => {
                let _ = line_editor.add_history_entry(line.as_str());
                let command = CommandParser::parse(&line, editor.in_multi_line, editor.edit_mode);
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