use art::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn ipv4_table<D>() -> ArtRoot<D> {
    // ArtRoot::new(7, [8u8, 4u8, 4u8, 4u8, 4u8, 4u8, 4u8].to_vec(), 32)
    ArtRoot::new(8, [4u8; 8].to_vec(), 32)
}

#[test]
fn v4route_random1() {
    let mut top = ipv4_table();
    top.route_ipv4_add("10.0.0.0/30", 30);

    let file = File::open("tests/data/v4routes-random1.txt").unwrap();
    let bufferd = BufReader::new(file);

    for line in bufferd.lines() {
        let line = line.unwrap();
        top.route_ipv4_add(&line, 0);
    }
    assert_eq!(top.iter().count(), 100);
}
