extern crate memchr;
extern crate walkdir;
extern crate crypto;

use std::env;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use walkdir::WalkDir;
use crypto::digest::Digest;
use crypto::md5::Md5;

struct StrictResult {
    filename: String,
    lineno: usize,
    col: usize,
    line: String,
}

const UNWRAP_METHOD: &str = ".unwrap()";

impl StrictResult {
    fn new(filename: &str, lineno: usize, col: usize, line: &str) -> Self {
        Self {
            filename: filename.to_string(),
            lineno: lineno,
            col: col,
            line: line.to_string(),
        }
    }

    fn gen_md5hash(&self) -> String {
        let mut md5 = Md5::new();
        let key = format!("{}_{}_{}_{}", self.filename, self.lineno, self.col, self.line);
        md5.input(key.as_bytes());
        md5.result_str().to_string()
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
    match line.find(UNWRAP_METHOD) {
        Some(col) => {
            if is_comment_or_string(col, col+UNWRAP_METHOD.len()-1, line) {
                None
            } else {
                Some(StrictResult::new(filename, lineno, col, line))
            }
        },
        None => None,
    }
}

fn exec_check(filename: &str) -> Vec<StrictResult> {
    let mut results = vec![];
    let input = File::open(filename).expect("fail open file");
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

fn exec_fix(result: &StrictResult) {
    let filename = &result.filename;
    let input = File::open(filename).expect("fail open file");
    let output_filename = format!("{}.strictfix", filename);
    let output = File::create(output_filename).expect("fail create file");
    let mut buf = BufReader::new(input);
    let mut wbuf = BufWriter::new(output);
    let mut line = String::new();
    let mut lineno: usize = 0;
    loop {
        if buf.read_line(&mut line).expect("read_line() error") <= 0 {
            break;
        }
        if lineno == (&result).lineno {
            let md5 = result.gen_md5hash();
            let ex = format!(".expect(\"error-id:{}\")", md5);
            let new_line = line.replace(UNWRAP_METHOD, ex.as_str());
            let _ = wbuf.write(new_line.as_bytes());
        } else {
            let _ = wbuf.write(line.as_bytes());
        }
        line.clear();
        lineno += 1;
    }
}

fn main() {
    // check fix mode
    let mut is_fix_mode = false;
    for arg in env::args() {
        if arg.as_str() == "--fix" {
            is_fix_mode = true;
        }
    }

    let args = env::args().skip(2);
    let mut args = args.filter(|x| x.as_str() != "--fix").collect::<Vec<String>>();

    // walk directory and collect filepath when non args.
    if args.len() == 0 {
        args = vec![];
        for entry in WalkDir::new("./") {
            let entry = entry.expect("$2");
            if !entry.file_type().is_file() {
                continue;
            }
            let filepath = entry.path().to_str().expect("#2");
            if !filepath.ends_with(".rs") {
                continue;
            }
            args.push(filepath.to_string());
        }
    }

    for arg in args {
        let results = exec_check(arg.as_str());
        for result in results {
            if is_fix_mode {
                exec_fix(&result);
            } else {
                println!("{}:{}:{}: {}", result.filename, result.lineno, result.col, result.line);
            }
        }
    }
}
