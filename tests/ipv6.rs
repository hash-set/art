use art::*;

fn ipv6_table<D>() -> ArtRoot<D> {
    ArtRoot::new(32, [4u8; 32].to_vec(), 128)
}

#[test]
fn ipv6_delete_default() {
    let mut top = ipv6_table();

    top.route_ipv6_add("::/0", 0);
    assert_eq!(top.iter().count(), 1);

    top.route_ipv6_delete("::/0");
    assert_eq!(top.iter().count(), 0);
}
