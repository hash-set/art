use art::*;
use ipnet::IpNet;

fn lookup_assert<D>(top: &ArtRoot<D>, addr: &str, route: &str) {
    let n = top.route_ipv4_lookup(addr);
    let p: IpNet = route.parse().unwrap();
    assert_eq!(n.unwrap().prefix, p);
}

fn lookup_assert_none<D>(top: &ArtRoot<D>, addr: &str) {
    let n = top.route_ipv4_lookup(addr);
    assert!(n.is_none());
}

fn lookup_test<D>(top: &ArtRoot<D>) {
    lookup_assert(top, "10.0.0.0/32", "10.0.0.0/32");
    lookup_assert(top, "10.0.0.1/32", "10.0.0.0/31");
    lookup_assert(top, "10.0.0.2/32", "10.0.0.0/30");
    lookup_assert(top, "10.0.0.3/32", "10.0.0.0/30");
    lookup_assert(top, "10.0.0.4/32", "10.0.0.0/29");
    lookup_assert(top, "10.0.0.7/32", "10.0.0.0/29");
    lookup_assert(top, "10.0.0.8/32", "10.0.0.0/28");
    lookup_assert(top, "10.0.0.15/32", "10.0.0.0/28");
    lookup_assert(top, "10.0.0.0/28", "10.0.0.0/28");
    lookup_assert_none(top, "10.0.0.16/32");
    lookup_assert_none(top, "10.0.0.255/32");
    lookup_assert_none(top, "0.0.0.0/0");
}

#[test]
fn ipv4_default_test() {
    let mut top = ArtRoot::new_ipv4_table();

    // Default route
    top.route_ipv4_add("0.0.0.0/0", 0);
    top.route_ipv4_add("0.0.0.1/32", 32);

    lookup_assert(&top, "0.0.0.0/8", "0.0.0.0/0");
    lookup_assert(&top, "10.10.10.10/32", "0.0.0.0/0");
    lookup_assert(&top, "0.0.0.1/32", "0.0.0.1/32");
}

#[test]
fn ipv4_lookup_single_test() {
    let mut top = ArtRoot::new_ipv4_table();

    // 10.0.0.0/24
    top.route_ipv4_add("10.0.0.0/24", 24);

    lookup_assert(&top, "10.0.0.128/32", "10.0.0.0/24");
    lookup_assert(&top, "10.0.0.1/32", "10.0.0.0/24");
    lookup_assert(&top, "10.0.0.0/25", "10.0.0.0/24");
    lookup_assert(&top, "10.0.0.0/24", "10.0.0.0/24");
    lookup_assert_none(&top, "10.0.0.0/23");
}

#[test]
fn ipv4_lookup_order_test() {
    let mut top = ArtRoot::new_ipv4_table();

    // 10.0.0.0/{28..32}
    top.route_ipv4_add("10.0.0.0/28", 28);
    top.route_ipv4_add("10.0.0.0/29", 29);
    top.route_ipv4_add("10.0.0.0/30", 30);
    top.route_ipv4_add("10.0.0.0/31", 31);
    top.route_ipv4_add("10.0.0.0/32", 32);

    lookup_test(&top);
}

#[test]
fn ipv4_lookup_reverse_test() {
    let mut top = ArtRoot::new_ipv4_table();

    // 10.0.0.0/{28..32}
    top.route_ipv4_add("10.0.0.0/32", 32);
    top.route_ipv4_add("10.0.0.0/31", 31);
    top.route_ipv4_add("10.0.0.0/30", 30);
    top.route_ipv4_add("10.0.0.0/29", 29);
    top.route_ipv4_add("10.0.0.0/28", 28);

    lookup_test(&top);
}

#[test]
fn ipv4_lookup_random_test() {
    let mut top = ArtRoot::new_ipv4_table();

    // 10.0.0.0/{28..32}
    top.route_ipv4_add("10.0.0.0/30", 30);
    top.route_ipv4_add("10.0.0.0/32", 32);
    top.route_ipv4_add("10.0.0.0/28", 28);
    top.route_ipv4_add("10.0.0.0/29", 29);
    top.route_ipv4_add("10.0.0.0/31", 31);

    lookup_test(&top);
}

#[test]
fn ipv4_inter_count() {
    let mut top = ArtRoot::new_ipv4_table();

    top.route_ipv4_add("0.0.0.0/0", 0);
    top.route_ipv4_add("0.0.0.0/1", 1);
    top.route_ipv4_add("128.0.0.0/1", 1);

    top.route_ipv4_add("0.0.0.0/2", 2);
    top.route_ipv4_add("64.0.0.0/2", 2);
    top.route_ipv4_add("128.0.0.0/2", 2);
    top.route_ipv4_add("192.0.0.0/2", 2);

    top.route_ipv4_add("0.0.0.0/3", 3);
    top.route_ipv4_add("32.0.0.0/3", 3);
    top.route_ipv4_add("64.0.0.0/3", 3);
    top.route_ipv4_add("96.0.0.0/3", 3);
    top.route_ipv4_add("128.0.0.0/3", 3);
    top.route_ipv4_add("160.0.0.0/3", 3);
    top.route_ipv4_add("192.0.0.0/3", 3);
    top.route_ipv4_add("224.0.0.0/3", 3);

    top.route_ipv4_add("0.0.0.0/4", 4);
    top.route_ipv4_add("32.0.0.0/4", 4);
    top.route_ipv4_add("64.0.0.0/4", 4);
    top.route_ipv4_add("96.0.0.0/4", 4);
    top.route_ipv4_add("128.0.0.0/4", 4);
    top.route_ipv4_add("160.0.0.0/4", 4);
    top.route_ipv4_add("192.0.0.0/4", 4);
    top.route_ipv4_add("224.0.0.0/4", 4);
    top.route_ipv4_add("16.0.0.0/4", 4);
    top.route_ipv4_add("48.0.0.0/4", 4);
    top.route_ipv4_add("89.0.0.0/4", 4);
    top.route_ipv4_add("112.0.0.0/4", 4);
    top.route_ipv4_add("144.0.0.0/4", 4);
    top.route_ipv4_add("176.0.0.0/4", 4);
    top.route_ipv4_add("208.0.0.0/4", 4);
    top.route_ipv4_add("240.0.0.0/4", 4);

    assert_eq!(top.iter().count(), 31);
}

#[test]
fn ipv4_delete_default() {
    let mut top = ArtRoot::new_ipv4_table();

    top.route_ipv4_add("0.0.0.0/0", 0);
    assert_eq!(top.iter().count(), 1);

    top.route_ipv4_delete("0.0.0.0/0");
    assert_eq!(top.iter().count(), 0);
}

#[test]
fn ipv4_delete_table_default() {
    let mut top = ArtRoot::new_ipv4_table();

    top.route_ipv4_add("0.0.0.0/4", 4);
    assert_eq!(top.iter().count(), 1);

    top.route_ipv4_add("0.0.0.0/5", 5);
    assert_eq!(top.iter().count(), 2);

    top.route_ipv4_delete("0.0.0.0/4");
    assert_eq!(top.iter().count(), 1);
}
