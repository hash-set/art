use ipnet::IpNet;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct ArtRoot<D> {
    bits: Vec<u8>,
    levels: u32,
    alen: u8,
    root: Option<Rc<ArtTable<D>>>,
}

impl<D> ArtRoot<D> {
    pub fn new(levels: u32, bits: Vec<u8>, alen: u8) -> Self {
        let mut ar = ArtRoot {
            levels,
            bits,
            alen,
            root: None,
        };
        let mut plen = 0u8;
        for i in 0..levels {
            if (i as usize) < ar.bits.len() {
                plen += ar.bits[i as usize];
            }
        }
        assert!(plen == alen);
        ar.levels = levels;
        ar.root = Some(ArtTable::new(&ar, None, 0));
        ar
    }

    pub fn root(&self) -> Rc<ArtTable<D>> {
        self.root.as_ref().unwrap().clone()
    }

    pub fn insert(&mut self, an: &Rc<ArtEntry<D>>, prefix: &IpNet) {
        if prefix.prefix_len() > self.alen {
            return;
        }

        let mut at = self.root();

        if prefix.prefix_len() == 0 {
            at.set_default(an.clone());
            return;
        }

        while prefix.prefix_len() > at.offset + at.bits {
            let j = art_findex(&at, prefix).unwrap();

            let entry = at.entry[j as usize].borrow().clone();

            match entry.as_ref() {
                ArtEntry::Table(table) => {
                    at = table.clone();
                }
                ArtEntry::Node(node) => {
                    let table = ArtTable::new(self, Some(at.clone()), j);
                    table.set_default(ArtEntry::from_node(node.clone()));
                    at.set_entry(j, ArtEntry::from_table(table.clone()));
                    at = table.clone();
                }
                ArtEntry::None => {
                    let table = ArtTable::new(self, Some(at.clone()), j);
                    at.set_entry(j, Rc::new(ArtEntry::Table(table.clone())));
                    at = table.clone();
                }
            }
        }

        let i = art_bindex(&at, prefix, prefix.prefix_len()).unwrap();

        self.table_insert(&at, i, an.clone());
    }

    fn table_insert(&mut self, at: &ArtTable<D>, i: u32, an: Rc<ArtEntry<D>>) {
        let mut prev = at.get_entry(i);

        if ArtEntry::check_duplicate(&prev, &an) {
            return;
        }

        // If the index `i' of the route that we are inserting is not a fringe
        // index, we need to allot this new route pointer to all the fringe
        // indices.
        if i < at.minfringe {
            if let ArtEntry::Table(table) = prev.as_ref() {
                prev = table.get_default();
            }
            art_allot(at, i, prev, &an);
        } else if let ArtEntry::Table(table) = prev.as_ref() {
            table.set_default(an.clone());
        } else {
            at.set_entry(i, an.clone())
        }
    }

    pub fn lookup(&self, prefix: &IpNet) -> Option<Rc<ArtNode<D>>> {
        let mut at = self.root();
        let mut default = at.get_default();

        while prefix.prefix_len() > at.offset + at.bits {
            let j = art_findex(&at, prefix).unwrap();
            let entry = at.entry[j as usize].borrow().clone();

            match entry.as_ref() {
                ArtEntry::Table(table) => {
                    at = table.clone();
                    if at.has_default() {
                        default = at.get_default();
                    }
                }
                ArtEntry::Node(node) => {
                    return Some(node.clone());
                }
                ArtEntry::None => {
                    if let ArtEntry::Node(node) = default.as_ref() {
                        return Some(node.clone());
                    } else {
                        return None;
                    }
                }
            }
        }

        let i = art_bindex(&at, prefix, prefix.prefix_len()).unwrap();
        let entry = at.get_entry(i);

        match entry.as_ref() {
            ArtEntry::Node(node) => {
                return Some(node.clone());
            }
            ArtEntry::Table(table) => {
                if let ArtEntry::Node(node) = table.get_default().as_ref() {
                    return Some(node.clone());
                }
            }
            ArtEntry::None => {}
        }
        if let ArtEntry::Node(node) = default.as_ref() {
            return Some(node.clone());
        }
        None
    }

    pub fn lookup_exact(&self, prefix: &IpNet) -> Option<Rc<ArtNode<D>>> {
        let mut at = self.root();

        while prefix.prefix_len() > at.offset + at.bits {
            let j = art_findex(&at, prefix).unwrap();
            let entry = at.entry[j as usize].borrow().clone();

            match entry.as_ref() {
                ArtEntry::Table(table) => {
                    at = table.clone();
                }
                ArtEntry::Node(_) | ArtEntry::None => {
                    return None;
                }
            }
        }

        let i = art_bindex(&at, prefix, prefix.prefix_len()).unwrap();
        let entry = at.get_entry(i);

        match entry.as_ref() {
            ArtEntry::Node(node) => {
                if node.prefix.prefix_len() == prefix.prefix_len() {
                    return Some(node.clone());
                }
            }
            ArtEntry::Table(table) => {
                if let ArtEntry::Node(node) = table.get_default().as_ref() {
                    if node.prefix.prefix_len() == prefix.prefix_len() {
                        return Some(node.clone());
                    }
                }
            }
            ArtEntry::None => {}
        }
        None
    }

    pub fn delete(&mut self, prefix: &IpNet) {
        if prefix.prefix_len() > self.alen {
            return;
        }

        let mut at = self.root();

        while prefix.prefix_len() > at.offset + at.bits {
            let j = art_findex(&at, prefix).unwrap();
            let entry = at.entry[j as usize].borrow().clone();

            match entry.as_ref() {
                ArtEntry::Table(table) => {
                    at = table.clone();
                }
                ArtEntry::Node(_) | ArtEntry::None => {
                    return;
                }
            }
        }

        let i = art_bindex(&at, prefix, prefix.prefix_len()).unwrap();
        let mut prev = at.get_entry(i);

        if i < at.minfringe {
            if let ArtEntry::Table(table) = prev.as_ref() {
                prev = table.get_default();
            }
            art_allot(&at, i, prev, &ArtEntry::none());
        } else {
            match prev.as_ref() {
                ArtEntry::Table(table) => {
                    if table.has_default() {
                        table.set_default(ArtEntry::none());
                    }
                }
                ArtEntry::Node(_) => {
                    at.set_entry(i, ArtEntry::none());
                }
                ArtEntry::None => {}
            }
        }
    }

    pub fn iter(&self) -> ArtIter<D> {
        ArtIter {
            i: 1,
            at: self.root(),
        }
    }

    pub fn route_ipv4_add(&mut self, str: &str, data: D) {
        let prefix: IpNet = str.parse().unwrap();
        let node = Rc::new(ArtEntry::Node(Rc::new(ArtNode {
            data: Some(data),
            prefix,
        })));
        self.insert(&node, &prefix);
    }

    pub fn route_ipv4_delete(&mut self, str: &str) {
        let prefix: IpNet = str.parse().unwrap();
        self.delete(&prefix);
    }

    pub fn route_ipv4_lookup(&self, str: &str) -> Option<Rc<ArtNode<D>>> {
        let prefix: IpNet = str.parse().unwrap();
        self.lookup(&prefix)
    }

    pub fn route_ipv4_lookup_exact(&self, str: &str) -> Option<Rc<ArtNode<D>>> {
        let prefix: IpNet = str.parse().unwrap();
        self.lookup_exact(&prefix)
    }

    pub fn route_ipv6_add(&mut self, str: &str, data: D) {
        self.route_ipv4_add(str, data);
    }

    pub fn route_ipv6_delete(&mut self, str: &str) {
        self.route_ipv4_delete(str);
    }

    pub fn route_ipv6_lookup(&self, str: &str) -> Option<Rc<ArtNode<D>>> {
        self.route_ipv4_lookup(str)
    }

    pub fn route_ipv6_lookup_exact(&self, str: &str) -> Option<Rc<ArtNode<D>>> {
        self.route_ipv4_lookup_exact(str)
    }
}

#[derive(Default)]
pub struct ArtTable<D> {
    minfringe: u32,
    pub level: u32,
    index: u32,
    bits: u8,
    offset: u8,
    parent: Option<Rc<ArtTable<D>>>,
    entry: Vec<RefCell<Rc<ArtEntry<D>>>>,
}

impl<D> ArtTable<D> {
    fn new(root: &ArtRoot<D>, parent: Option<Rc<ArtTable<D>>>, j: u32) -> Rc<Self> {
        let mut table = ArtTable {
            minfringe: 0,
            level: 0,
            index: j,
            bits: 0,
            offset: 0,
            parent: parent.clone(),
            entry: Vec::new(),
        };
        let level = if let Some(parent) = parent.as_ref() {
            table.offset = parent.offset + parent.bits;
            parent.level + 1
        } else {
            0
        };
        table.minfringe = 1 << root.bits[level as usize];
        table.level = level;
        table.bits = root.bits[level as usize];
        table.entry = vec![RefCell::new(Rc::new(ArtEntry::None)); (table.minfringe << 1) as usize];
        Rc::new(table)
    }

    pub fn get_entry(&self, i: u32) -> Rc<ArtEntry<D>> {
        self.entry[i as usize].borrow().clone()
    }

    fn set_entry(&self, i: u32, an: Rc<ArtEntry<D>>) {
        self.entry[i as usize].replace(an);
    }

    fn has_default(&self) -> bool {
        matches!(self.entry[1].borrow().as_ref(), ArtEntry::Node(_))
    }

    fn get_default(&self) -> Rc<ArtEntry<D>> {
        self.entry[1].borrow().clone()
    }

    fn set_default(&self, an: Rc<ArtEntry<D>>) {
        self.entry[1].replace(an);
    }
}

pub struct ArtIter<D> {
    at: Rc<ArtTable<D>>,
    i: usize,
}

impl<D> IntoIterator for &ArtRoot<D> {
    type Item = Rc<ArtNode<D>>;
    type IntoIter = ArtIter<D>;

    fn into_iter(self) -> Self::IntoIter {
        ArtIter {
            i: 1,
            at: self.root(),
        }
    }
}

impl<D> Iterator for ArtIter<D> {
    type Item = Rc<ArtNode<D>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < (self.at.minfringe << 1) as usize {
            let entry = self.at.get_entry(self.i as u32);
            match entry.as_ref() {
                ArtEntry::Node(node) => {
                    if let Some(j) = art_bindex(&self.at, &node.prefix, node.prefix.prefix_len()) {
                        if self.i == j as usize {
                            self.i += 1;
                            return Some(node.clone());
                        }
                    }
                }
                ArtEntry::Table(table) => {
                    self.at = table.clone();
                    self.i = 1;
                    return self.next();
                }
                _ => {}
            }
            self.i += 1;
        }
        if let Some(parent) = &self.at.parent {
            self.i = (self.at.index + 1) as usize;
            self.at = parent.clone();
            return self.next();
        }
        None
    }
}

#[derive(Debug)]
pub struct ArtNode<D> {
    pub prefix: IpNet,
    pub data: Option<D>,
}

impl<D> ArtNode<D> {
    pub fn new(prefix: &IpNet, data: Option<D>) -> Rc<Self> {
        Rc::new(Self {
            prefix: *prefix,
            data,
        })
    }
}

#[derive(Default)]
pub enum ArtEntry<D> {
    Table(Rc<ArtTable<D>>),
    Node(Rc<ArtNode<D>>),
    #[default]
    None,
}

impl<D> ArtEntry<D> {
    fn none() -> Rc<ArtEntry<D>> {
        Rc::new(ArtEntry::None)
    }

    pub fn from_node(node: Rc<ArtNode<D>>) -> Rc<ArtEntry<D>> {
        Rc::new(ArtEntry::Node(node))
    }

    pub fn from_table(node: Rc<ArtTable<D>>) -> Rc<ArtEntry<D>> {
        Rc::new(ArtEntry::Table(node))
    }

    fn check_duplicate(old: &Rc<ArtEntry<D>>, new: &Rc<ArtEntry<D>>) -> bool {
        std::ptr::eq(old.as_ref(), new.as_ref())
    }
}

pub fn to_octets(ipnet: &IpNet) -> Vec<u8> {
    match ipnet {
        IpNet::V4(v4net) => v4net.addr().octets().to_vec(),
        IpNet::V6(v6net) => v6net.addr().octets().to_vec(),
    }
}

// Return the base index of the part of ``addr'' and ``plen''
// corresponding to the range covered by the table ``at''.
//
// In other words, this function take the multi-level (complete)
// address ``addr'' and prefix length ``plen'' and return the
// single level base index for the table ``at''.
//
// For example with an address size of 32bit divided into four
// 8bit-long tables, there's a maximum of 4 base indexes if the
// prefix length is > 24.
//
fn art_bindex<D>(at: &ArtTable<D>, prefix: &IpNet, mut plen: u8) -> Option<u32> {
    let mut k: u32;
    //let mut plen = prefix.prefix_len();

    if plen < at.offset || plen > (at.offset + at.bits) {
        return None;
    }

    // We are only interested in the part of the prefix length
    // corresponding to the range of this table.
    plen -= at.offset;

    // Jump to the first byte of the address containing bits
    // covered by this table.
    let addr = to_octets(prefix);
    let offset: usize = (at.offset / 8) as usize;

    // ``at'' covers the bit range between ``boff'' & ``bend''. */
    let boff = at.offset % 8;
    let bend = at.bits + boff;

    if bend > 24 {
        k = ((addr[offset] as u32) & ((1 << (8 - boff)) - 1)) << (bend - 8);
        k |= (addr[offset + 1] as u32) << (bend - 16);
        k |= (addr[offset + 2] as u32) << (bend - 24);
        k |= (addr[offset + 3] as u32) >> (32 - bend);
    } else if bend > 16 {
        k = ((addr[offset] as u32) & ((1 << (8 - boff)) - 1)) << (bend - 8);
        k |= (addr[offset + 1] as u32) << (bend - 16);
        k |= (addr[offset + 2] as u32) >> (24 - bend);
    } else if bend > 8 {
        k = ((addr[offset] as u32) & ((1 << (8 - boff)) - 1)) << (bend - 8);
        k |= (addr[offset + 1] as u32) >> (16 - bend);
    } else {
        k = ((addr[offset] as u32) >> (8 - bend)) & ((1 << at.bits) - 1);
    }

    Some((k >> (at.bits - plen)) + (1 << plen))
}

fn art_findex<D>(at: &ArtTable<D>, prefix: &IpNet) -> Option<u32> {
    art_bindex(at, prefix, at.offset + at.bits)
}

fn art_allot<D>(at: &ArtTable<D>, i: u32, old: Rc<ArtEntry<D>>, new: &Rc<ArtEntry<D>>) {
    let mut k = i;

    let exist = at.get_entry(k);
    match exist.as_ref() {
        ArtEntry::Table(table) => {
            let default = table.get_default();
            if std::ptr::eq(default.as_ref(), old.as_ref()) {
                table.set_default(new.clone());
            }
        }
        ArtEntry::Node(_) => {
            if std::ptr::eq(exist.as_ref(), old.as_ref()) {
                at.set_entry(k, new.clone());
            }
        }
        ArtEntry::None => {
            at.set_entry(k, new.clone());
        }
    }

    if k >= at.minfringe {
        return;
    }

    k <<= 1;
    art_allot(at, k, old.clone(), &new.clone());
    k += 1;
    art_allot(at, k, old.clone(), &new.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_art_bindex() {
        let ar = ArtRoot::<u32>::new(8, [4u8; 8].to_vec(), 32);
        let at = ArtTable::new(&ar, None, 0);

        let net0: IpNet = "0.0.0.0/0".parse().unwrap();
        let bindex = art_bindex(&at, &net0, net0.prefix_len()).unwrap();
        assert_eq!(bindex, 1);

        let net0: IpNet = "0.0.0.0/1".parse().unwrap();
        let bindex = art_bindex(&at, &net0, net0.prefix_len()).unwrap();
        assert_eq!(bindex, 2);

        let net128: IpNet = "128.0.0.0/1".parse().unwrap();
        let bindex = art_bindex(&at, &net128, net128.prefix_len()).unwrap();
        assert_eq!(bindex, 3);

        let net128: IpNet = "128.0.0.0/4".parse().unwrap();
        let bindex = art_bindex(&at, &net128, net128.prefix_len()).unwrap();
        assert_eq!(bindex, 24);

        let net224: IpNet = "224.0.0.0/3".parse().unwrap();
        let bindex = art_bindex(&at, &net224, net224.prefix_len()).unwrap();
        assert_eq!(bindex, 15);

        let net240: IpNet = "240.0.0.0/4".parse().unwrap();
        let bindex = art_bindex(&at, &net240, net240.prefix_len()).unwrap();
        assert_eq!(bindex, 31);
    }
}
