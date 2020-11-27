use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub fn read_first_line(path: PathBuf) -> Option<String> {
    let file = File::open(path).unwrap();
    let buffer = BufReader::new(file);
    let mut lines_iter = buffer.lines().map(|r| r.unwrap());
    return lines_iter.next();
}
