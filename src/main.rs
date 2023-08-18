use art::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

fn ipv4_table<D>() -> ArtRoot<D> {
    ArtRoot::new(8, [4u8; 8].to_vec(), 32)
}

#[allow(dead_code)]
fn ipv4_lookup_order_test() {
    let mut top = ipv4_table();

    // 10.0.0.0/{28..32}
    top.route_ipv4_add("10.0.0.0/28", 28);
    top.route_ipv4_add("10.0.0.0/29", 29);
    top.route_ipv4_add("10.0.0.0/30", 30);
    top.route_ipv4_add("10.0.0.0/31", 31);
    top.route_ipv4_add("10.0.0.0/32", 32);

    println!("{}", top.iter().count());
}

pub fn dive<D>(at: Rc<ArtTable<D>>) {
    println!("Table: {}", at.level);
    for i in 1..32 {
        println!("table:i: {}:{}", at.level, i);
        match at.get_entry(i).as_ref() {
            ArtEntry::Table(table) => {
                println!("Table:{} -> {}", i, table.level);
                dive(table.clone());
            }
            ArtEntry::Node(node) => {
                if i == 1 {
                    println!("Default {} level {}", node.prefix, at.level);
                }
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
fn v4route_random1() {
    let mut top = ipv4_table();
    top.route_ipv4_add("10.0.0.0/30", 30);

    let file = File::open("tests/data/v4routes-random1.txt").unwrap();
    let bufferd = BufReader::new(file);

    for line in bufferd.lines() {
        let line = line.unwrap();
        top.route_ipv4_add(&line, 0);
    }
    dive(top.root());

    // assert_eq!(top.iter().count(), 100);
}

fn test() {
    let mut top = ipv4_table();

    top.route_ipv4_add("1.0.192.0/19", 19);
    top.route_ipv4_add("1.0.204.0/22", 22);
}

fn main() {
    test();
    // ipv4_lookup_order_test();
    // v4route_random1();
}
