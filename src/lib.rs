use std::net::Ipv4Addr;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidNetwork,
    CidrMissMatch
}

#[derive(Debug, PartialEq)]
pub enum IpNetwork {
    V4(Ipv4Network)
}

#[derive(Debug)]
pub struct Ipv4Network {
    pub first: u32,
    pub cidr: u32
}

#[derive(Debug)]
pub struct NetworkIterator {
    current: u32,
    max: u32,
    stepping: u32,
    cidr: u32
}


#[derive(Debug)]
pub struct HostIterator {
    current: u32
}

#[inline(always)]
fn cidr_to_hostcount(cidr: u32) -> u32 {
    1 << (32 - cidr)
}

impl Ipv4Network {
    pub fn new(first: u32, cidr: u32) -> Result<Ipv4Network, Error> {
        match Ipv4Network::is_valid(first, cidr) {
            true => Ok(Ipv4Network {first: first, cidr: cidr}),
            false => Err(Error::InvalidNetwork)
        }
    }
    pub fn hostcount(&self) -> u32 {
        cidr_to_hostcount(self.cidr)
    }
    pub fn subnets(&self, new_cidr: u32) -> NetworkIterator {
        NetworkIterator {
            current: self.first,
            stepping: cidr_to_hostcount(new_cidr),
            cidr: new_cidr,
            max: self.first + self.hostcount() - 1
        }
    }
    pub fn last(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.first + self.hostcount() - 1)
    }
    pub fn first(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.first)
    }
    pub fn contains(&self, ip_addr: &Ipv4Addr) -> bool {
        let ip_int = u32::from(ip_addr.clone());
        ip_int > self.first && ip_int < (self.first + self.hostcount() - 1)
    }

    #[inline(always)]
    fn is_valid(first: u32, cidr: u32) -> bool {
        first % cidr_to_hostcount(cidr) == 0
    }
}

impl Iterator for NetworkIterator {
    type Item = Ipv4Network;
    fn next(&mut self) -> Option<Ipv4Network> {
        if self.current <  self.max {
            self.current += self.stepping;
            match Ipv4Network::new(self.current, self.cidr) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            return None;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.max as usize, Some(self.max as usize))
    }
}

impl PartialEq for Ipv4Network {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.cidr == other.cidr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn new_network() {
        assert_eq!(
            Ok(Ipv4Network { first: 16843008, cidr: 25}),
            Ipv4Network::new(16843008, 25)
        );
    }
    #[test]
    fn new_network_invalid() {
        assert_eq!(
            Err(Error::InvalidNetwork),
            Ipv4Network::new(16843008, 23)
        );
    }
    #[test]
    fn first_address() {
        let network = Ipv4Network::new(16843008, 24).unwrap();
        let first: Ipv4Addr = "1.1.1.0".parse().unwrap();
        assert_eq!(first, network.first());

    }
    #[test]
    fn last_address() {
        let network = Ipv4Network::new(16843008, 24).unwrap();
        let last: Ipv4Addr = Ipv4Addr::from_str("1.1.1.255").unwrap();
        assert_eq!(last, network.last());
    }
    #[test]
    fn contains_addr() {
        let network = Ipv4Network::new(16843008, 24).unwrap();
        assert_eq!(network.contains(&Ipv4Addr::new(1,1,1,1)), true);
    }
    #[test]
    fn iterate() {
        let network = Ipv4Network::new(16843008, 24).unwrap();
        let test = network.subnets(25);
        assert_eq!(test.stepping, 128);
        let test2: Vec<Ipv4Network> = test.collect();
        assert_eq!(test2.len(), 2);
    }
    #[test]
    fn test_long_iter() {
        let network = Ipv4Network::new(16777216, 8).unwrap();
        let networks: Vec<Ipv4Network> = network.subnets(32).collect();
        assert_eq!(networks[0].cidr, 32);
    }
}
