#![forbid(unsafe_code)]

use std::env::args;
use std::io::BufRead;
use std::{collections::HashSet, fs::File, io::BufReader};
fn main() {
    let args = args().collect::<Vec<String>>();

    let file1 = File::open(&args[1]).unwrap();
    let file2 = File::open(&args[2]).unwrap();

    let reader = BufReader::new(file1);

    let mut hashset = HashSet::new();
    let mut out_hash = HashSet::new();

    for line in reader.lines() {
        let string = line.unwrap();
        hashset.insert(string.clone());
    }

    let reader = BufReader::new(file2);

    for line in reader.lines() {
        let string = line.unwrap();
        if hashset.contains(&string) && !out_hash.contains(&string) {
            let tmp = string.clone();
            out_hash.insert(tmp.clone());
            println!("{}", tmp);
        }
    }
}
