extern crate csv;

use std::fs::File;

use csv::*;


fn main() {
    let mut reader = csv::Reader::from_reader(File::open("test.csv").unwrap());

    if !reader.has_headers() {
        panic!("");
    }

    for result in reader.records() {
        println!("{:?}", result);
    }
}

