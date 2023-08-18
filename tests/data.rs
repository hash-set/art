use art::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn ipv4_route_random1() {
    let mut top = ArtRoot::new_ipv4_table();

    let file = File::open("tests/data/v4routes-random1.txt").unwrap();
    let bufferd = BufReader::new(file);

    for line in bufferd.lines() {
        let line = line.unwrap();
        top.route_ipv4_add(&line, 0);
    }
    assert_eq!(top.iter().count(), 569770);
}

#[test]
#[ignore]
fn ipv4_route_random1_delete() {
    let mut top = ArtRoot::new_ipv4_table();

    let file = File::open("tests/data/v4routes-random1.txt").unwrap();
    let bufferd = BufReader::new(file);

    for line in bufferd.lines() {
        let line = line.unwrap();
        top.route_ipv4_add(&line, 0);
    }
    assert_eq!(top.iter().count(), 569770);

    let file = File::open("tests/data/v4routes-random1.txt").unwrap();
    let bufferd = BufReader::new(file);

    for line in bufferd.lines() {
        let line = line.unwrap();
        top.route_ipv4_delete(&line);
    }

    assert_eq!(top.iter().count(), 0);
}
