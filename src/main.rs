extern crate core;
extern crate colored;

use std::process::Command;
use std::string::ToString;
use bat::line_range::{LineRange, LineRanges};
use serde::{Deserialize, Serialize};
use bat::PrettyPrinter;
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug)]
enum ErrKind {
    #[serde(rename = "note")]
    Note,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
}

#[derive(Serialize, Deserialize, Debug)]
struct SrcPos {
    file: String,
    line: i32,
    column: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SrcLocation {
    caret: SrcPos,
}

#[derive(Serialize, Deserialize, Debug)]
struct GccErr {
    kind: ErrKind,
    locations: Vec<SrcLocation>,
    message: String
}

impl GccErr {

    pub fn from(s: &str) -> serde_json::Result<Vec<GccErr>> {
        serde_json::from_str(s) as Result<Vec<GccErr>, _>
    }

    pub fn render(&self) {

        println!("[{}] {}", match &self.kind {
            ErrKind::Note => "Note".cyan().bold(),
            ErrKind::Error => "Error".red().bold(),
            ErrKind::Warning => "Warning".yellow().bold(),
        }, self.message);

        for src_loc in self.locations.iter() {

            let caret = &src_loc.caret;
            println!(" File `{}` line {} col {}:",
                caret.file.green(),
                caret.line.to_string().cyan(),
                caret.column.to_string().cyan(),
            );

            let line_from = if caret.line < 3 { 0 } else { caret.line - 3 } as usize;
            let line_to = (caret.line + 3) as usize;

            let mut printer = PrettyPrinter::new();
            printer
                .input_file(&caret.file)
                .header(false)
                .line_numbers(true)
                .line_ranges(LineRanges::from(vec![
                    LineRange::new(line_from, line_to)
                ]))
                .highlight(caret.line as usize);
            printer.print().unwrap();
        }
        println!();
    }
}

fn main() {
    let output = Command::new("g++")
        .args(std::env::args().skip(1))
        .arg("-fdiagnostics-format=json")
        .output()
        .expect("Unable to start gcc");

    let stderr = &String::from_utf8(output.stderr).unwrap();
    let stdout = &String::from_utf8(output.stdout).unwrap();

    match GccErr::from(stderr) {
        Ok(gcc_output) => gcc_output.iter().for_each(|it| it.render()),
        Err(_) => {
            eprint!("{}", stderr);
            print!("{}", stdout);
        }
    }
}
