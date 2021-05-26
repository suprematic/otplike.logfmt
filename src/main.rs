use chrono::prelude::*;
use std::io::{self, BufRead};

use ansi_term::Colour;

use serde_json::map::Map;
use serde_json::{self, Value};

use colored_json::to_colored_json_auto;

use atty::Stream;

trait Format {
    fn format<T: Formatter>(&self) -> String;
}

impl Format for Option<&str> {
    fn format<T: Formatter>(&self) -> String {
        T::format(*self)
    }
}

trait Formatter {
    fn format(s: Option<&str>) -> String;
}

struct LevelFormatter;
impl Formatter for LevelFormatter {
    fn format(level: Option<&str>) -> String {
        let level = level.or(Some("XXXXXX")).unwrap();

        let highlight = match level {
            "alert" | "critical" | "error" => Colour::Red,
            "warning" | "notice" => Colour::Yellow,
            "info" => Colour::Blue,
            "debug" => Colour::Purple,
            _ => Colour::Blue,
        };

        highlight.paint(format!("[{:^7}]", level)).to_string()
    }
}

struct WhenFormatter;
impl Formatter for WhenFormatter {
    fn format(when: Option<&str>) -> String {
        let when = if let Some(when) = when {
            when.parse::<DateTime<Local>>()
                .and_then(|when| Ok(when.format("%Y-%m-%d %H:%M:%S").to_string()))
                .unwrap()
        } else {
            "XXXX-XX-XX XX:XX:XX".to_string()
        };

        Colour::Blue.paint(when).to_string()
    }
}

struct PidFormatter;
impl Formatter for PidFormatter {
    fn format(pid: Option<&str>) -> String {
        let pid = pid.or(Some("XXXXXX")).unwrap();
        Colour::Blue.paint(format!("{:<10}", pid)).to_string()
    }
}

struct WhatFormatter;
impl Formatter for WhatFormatter {
    fn format(what: Option<&str>) -> String {
        let what = what.or(Some("")).unwrap();

        Colour::White.paint(what).to_string()
    }
}

struct InFormatter;
impl Formatter for InFormatter {
    fn format(in_: Option<&str>) -> String {
        let in_ = in_.or(Some("")).unwrap();

        Colour::White.paint(format!("| {}", in_)).to_string()
    }
}

struct TextFormatter;
impl Formatter for TextFormatter {
    fn format(text: Option<&str>) -> String {
        format!(
            "{} {}",
            Colour::Blue.bold().paint(">>"),
            Colour::Green.paint(text.unwrap())
        )
    }
}

fn process_line(mut line: Map<String, Value>) {
    {
        let when = line
            .get("when")
            .and_then(Value::as_str)
            .format::<WhenFormatter>();

        let level = line
            .get("level")
            .and_then(Value::as_str)
            .format::<LevelFormatter>();

        let pid = line
            .get("pid")
            .and_then(Value::as_str)
            .format::<PidFormatter>();

        let what = line
            .get("what")
            .and_then(Value::as_str)
            .format::<WhatFormatter>();

        let in_ = line
            .get("in")
            .and_then(Value::as_str)
            .format::<InFormatter>();

        println!("{} {} {} {} {}", when, level, pid, in_, what);

        let text = line.get("text").and_then(Value::as_str);
        if text.is_some() {
            println!("{}", text.format::<TextFormatter>());
        }
    }

    for k in vec![
        "when", "level", "pid", "what", "in", "at", "log", "id", "text",
    ] {
        line.remove(k);
    }

    if !line.is_empty() {
        let highlighted = to_colored_json_auto(&Value::Object(line));
        println!("{}\n", highlighted.unwrap());
    }
}

fn main() -> io::Result<()> {
    let tty = atty::is(Stream::Stdout);

    #[cfg(windows)]
    if tty {
        colored_json::enable_ansi_support();
    }

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let ref line = line?;

        if tty {
            let parsed = serde_json::from_str(line);

            match parsed {
                Ok(serde_json::Value::Object(line)) => {
                    process_line(line);
                }

                Err(e) => {
                    println!(
                        "{}",
                        Colour::Red.paint(format!("cannot parse log line: {}\n{}", e, line))
                    );
                }

                _ => {}
            }
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}
