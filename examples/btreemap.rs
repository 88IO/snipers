#![feature(btree_drain_filter)]
use std::collections::BTreeMap;

fn main() {
    let mut map = BTreeMap::new();
    map.insert(3, "c");
    map.insert(2, "b");
    map.insert(1, "a");

    /*
    for (key, value) in map.drain_filter(|&k, _| k == 3) {
        println!("{}: {}", key, value);
    }
    */

    println!("{}", (&String::from("a"), 1) == (&String::from("a"), 1));

    println!("{:?}", map);
}
