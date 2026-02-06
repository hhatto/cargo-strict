use chrono::prelude::*;
use std::env;
use std::fs::{File, metadata};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use walkdir::WalkDir;

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
            lineno,
            col,
            line: line.to_string(),
        }
    }

    fn gen_md5hash(&self) -> md5::Digest {
        let key = format!(
            "{}_{}_{}_{}",
            self.filename, self.lineno, self.col, self.line
        );
        md5::compute(key.as_bytes())
    }
}

fn is_comment_or_string(target_col: usize, target_col_end: usize, line: &str) -> bool {
    // check in comment
    if let Some(i) = memchr::memchr2(b'/', b'/', line.as_bytes())
        && (target_col + 1) > i
    {
        return true;
    }
    if let Some(i) = memchr::memchr2(b'/', b'*', line.as_bytes())
        && (target_col + 1) > i
    {
        return true;
    }

    // check in string
    let line_length = line.len();
    let mut offset = 0;
    let mut start = true;
    let mut start_col = 0;
    let mut string_set: Vec<(usize, usize)> = vec![];
    let bline = line.as_bytes();
    while let Some(v) = memchr::memchr(b'"', &(bline[offset..])) {
        if start {
            start = false;
        } else {
            string_set.push((start_col, offset + v));
            start = true
        }
        start_col += v;
        offset += v + 1;
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
            if is_comment_or_string(col, col + UNWRAP_METHOD.len() - 1, line) {
                None
            } else {
                Some(StrictResult::new(filename, lineno + 1, col, line))
            }
        }
        None => None,
    }
}

fn exec_check(filename: &str) -> Vec<StrictResult> {
    let mut results = vec![];
    let input = File::open(filename).unwrap_or_else(|_| panic!("fail open file={}", filename));
    let mut buf = BufReader::new(input);
    let mut line = String::new();
    let mut lineno: usize = 0;
    loop {
        match buf.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(e) => panic!("read_line() error: {}", e),
        }
        if let Some(v) = check_strict(filename, lineno, line.trim_end()) {
            results.push(v)
        }
        line.clear();
        lineno += 1;
    }
    results
}

fn file2vecstr(filename: &str) -> Vec<String> {
    let mut f = BufReader::new(File::open(filename).expect("file open error"));
    let mut line = String::new();
    let mut strs = vec![];
    loop {
        match f.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(e) => panic!("read_line() error: {}", e),
        }
        strs.push(line.clone());
        line.clear();
    }
    strs
}

fn exec_fix_or_diff(result: &StrictResult, is_diff_mode: bool) {
    let filename = &result.filename;
    let input = File::open(filename).unwrap_or_else(|_| panic!("fail open file={}", filename));
    let output_filename = format!("{}.strictfix", filename);
    let output = File::create(output_filename.as_str()).expect("fail create file");
    {
        let mut buf = BufReader::new(input);
        let mut wbuf = BufWriter::new(output);
        let mut line = String::new();
        let mut lineno: usize = 0;
        loop {
            match buf.read_line(&mut line) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                }
                Err(e) => panic!("read_line() error: {}", e),
            }
            if lineno == (result).lineno {
                let md5 = result.gen_md5hash();
                let ex = format!(".expect(\"error-id:{:x}\")", md5);
                let new_line = line.replacen(UNWRAP_METHOD, ex.as_str(), 1);
                let _ = wbuf.write(new_line.as_bytes());
            } else {
                let _ = wbuf.write(line.as_bytes());
            }
            line.clear();
            lineno += 1;
        }
    }

    if is_diff_mode {
        // print diff
        let org = file2vecstr(filename);
        let orgtime = {
            let orgmeta = metadata(filename).expect("get orgfile metadata error");
            let v: DateTime<Local> = DateTime::from(
                orgmeta
                    .modified()
                    .expect("get original file modified time error"),
            );
            v.to_rfc2822()
        };

        let fix = file2vecstr(output_filename.as_str());
        let fixtime = {
            let fixmeta = metadata(output_filename.as_str()).expect("get fixfile metadata error");
            let v: DateTime<Local> = DateTime::from(
                fixmeta
                    .modified()
                    .expect("get fixed file modified time error"),
            );
            v.to_rfc2822()
        };
        let diff = difflib::unified_diff(
            &org,
            &fix,
            filename,
            output_filename.as_str(),
            orgtime.as_str(),
            fixtime.as_str(),
            3,
        );
        for l in &diff {
            print!("{}", l);
        }

        // remove tmp file
        if let Err(e) = std::fs::remove_file(output_filename.as_str()) {
            println!("remove file error: {:?}", e)
        }
    } else if let Err(e) = std::fs::rename(output_filename.as_str(), filename) {
        println!("rename error: {:?}, {} to {}", e, output_filename, filename)
    }
}

fn main() {
    // check fix mode
    let mut is_fix_mode = false;
    let mut is_diff_mode = false;
    for arg in env::args() {
        if arg.as_str() == "--fix" {
            is_fix_mode = true;
        }
        if arg.as_str() == "--diff" {
            is_diff_mode = true;
        }
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            println!("usage: cargo strict [--fix|--diff] [FILE]");
            std::process::exit(0);
        }
    }

    if is_diff_mode && is_fix_mode {
        println!("usage: cargo strict [--fix|--diff] [FILE]");
        std::process::exit(-1);
    }

    let args = env::args().skip(2);
    let mut args = args
        .filter(|x| x.as_str() != "--fix" && x.as_str() != "--diff")
        .collect::<Vec<String>>();

    // walk directory and collect filepath when non args.
    if args.is_empty() {
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
            if is_fix_mode || is_diff_mode {
                exec_fix_or_diff(&result, is_diff_mode);
            } else {
                println!(
                    "{}:{}:{}: {}",
                    result.filename, result.lineno, result.col, result.line
                );
            }
        }
    }
}
