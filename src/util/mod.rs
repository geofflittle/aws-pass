use std::{
    fs::File,
    io::{self, BufRead, BufReader, LineWriter, Write},
    path::PathBuf,
    process,
};

pub fn read_first_line(path: &PathBuf) -> Option<String> {
    let file = File::open(path).unwrap();
    let buffer = BufReader::new(file);
    let mut lines_iter = buffer.lines().map(|l| l.unwrap());
    return lines_iter.next();
}

pub fn prompt_stdin_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut value = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut value).unwrap();
    value.trim_end().to_string()
}

pub fn prompt_non_empty_str(name: &str) -> String {
    let line = prompt_stdin_line(&format!("{}:", name));
    if line.is_empty() {
        fatal_println!("{} must be non-empty", name);
    }
    line
}

pub fn write_lines<'a, I>(path: &PathBuf, lines: I)
where
    I: IntoIterator<Item = &'a str>,
{
    let file = File::create(path).unwrap();
    let mut lw = LineWriter::new(file);
    lines.into_iter().for_each(|line| {
        lw.write_all(line.as_bytes()).unwrap();
    });
}
