extern crate memchr;
extern crate walkdir;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use walkdir::WalkDir;

struct StrictResult {
    filename: String,
    lineno: usize,
    col: usize,
    line: String,
}

impl StrictResult {
    fn new(filename: &str, lineno: usize, col: usize, line: &str) -> Self {
        Self {
            filename: filename.to_string(),
            lineno: lineno,
            col: col,
            line: line.to_string(),
        }
    }
}

fn is_comment_or_string(target_col: usize, target_col_end: usize, line: &str) -> bool {
    // check in comment
    match memchr::memchr2('/' as u8, '/' as u8, line.as_bytes()) {
        Some(i) => {
            if (target_col+1) > i {
                return true;
            }
        },
        None => {},
    }
    match memchr::memchr2('/' as u8, '*' as u8, line.as_bytes()) {
        Some(i) => {
            if (target_col+1) > i {
                return true;
            }
        },
        None => {},
    }

    // check in string
    let line_length = line.len();
    let mut offset = 0;
    let mut start = true;
    let mut start_col = 0;
    let mut string_set: Vec<(usize, usize)> = vec![];
    let bline = line.as_bytes();
    loop {
        match memchr::memchr('"' as u8, &(bline[offset..])) {
            Some(v) => {
                if start {
                    start = false;
                } else {
                    string_set.push((start_col, offset+v));
                    start = true
                }
                start_col += v;
                offset += v + 1;
            },
            None => break,
        }
        if offset >= line_length {
            break;
        }
    }
    for (s, e) in string_set {
        if target_col > s && target_col_end < e {
            return true;
        }
    }

    false
}

fn check_strict(filename: &str, lineno: usize, line: &str) -> Option<StrictResult> {
    match line.find(".unwrap()") {
        Some(col) => {
            if is_comment_or_string(col, col+".unwrap()".len()-1, line) {
                None
            } else {
                Some(StrictResult::new(filename, lineno, col, line))
            }
        },
        None => None,
    }
}

fn check(filename: &str) -> Vec<StrictResult> {
    let mut results = vec![];
    //let input = File::open(filename).expect("fail open file");
    let input = File::open(filename).unwrap();
    let mut buf = BufReader::new(input);
    let mut line = String::new();
    let mut lineno: usize = 0;
    loop {
        if buf.read_line(&mut line).expect("read_line() error") <= 0 {
            break;
        }
        match check_strict(filename, lineno, line.trim_right()) {
            Some(v) => results.push(v),
            None => {},
        }
        line.clear();
        lineno += 1;
    }
    results
}

fn main() {
    let args = env::args().skip(2);

    if args.len() == 0 {
        for entry in WalkDir::new("./") {
            let entry = entry.expect("$2");
            if !entry.file_type().is_file() {
                continue;
            }
            let filepath = entry.path().to_str().expect("#2");
            if !filepath.ends_with(".rs") {
                continue;
            }
            let results = check(filepath);
            for result in results {
                println!("{}:{}:{}: {}", result.filename, result.lineno, result.col, result.line);
            }
        }
    } else {
        for arg in args {
            let results = check(arg.as_str());
            for result in results {
                println!("{}:{}:{}: {}", result.filename, result.lineno, result.col, result.line);
            }
        }
    }
}
