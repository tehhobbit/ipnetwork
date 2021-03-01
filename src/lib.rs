use std::net::Ipv4Addr;
use std::cmp::Ordering;
use std::str::FromStr;
use std::result::Result;
use std::convert::From;


#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidNetwork,
    CidrMissMatch,
    NetworkParseError,
}
#[derive(Debug, PartialEq)]
pub enum IpNetwork {
    V4(Ipv4Network)
}

/// An IPv4 network. The network is represented by
/// Network consists of a network address and cidr
/// the network address is represented as is represented
/// by u32
///
/// ```
/// use ipnetwork::Ipv4Network;
/// let network = "1.1.1.0/24".parse();
/// assert_eq!(Ok(Ipv4Network{first: 16843008, cidr: 24}), network)
#[derive(Debug, Eq)]
pub struct Ipv4Network {
    pub first: u32,
    pub cidr: u8
}
/// Iterator to iterate over subnets of a network
/// ```
/// use ipnetwork::Ipv4Network;
/// let network: Ipv4Network = "1.0.0.0/24".parse().unwrap();
/// let subnets: Vec<Ipv4Network> = network.subnets(25).collect();
/// assert_eq!(subnets.len(), 2);
/// ```
#[derive(Debug)]
pub struct NetworkIterator {
    /// The current network address
    current: u32,
    /// Upper bounds
    max: u32,
    /// How many addresses should the new network have
    stepping: u32,
    /// Cidr of the new network
    cidr: u8
}


#[derive(Debug)]
pub struct HostIterator {
    current: u32,
    max: u32
}

#[inline(always)]
fn cidr_to_hostcount(cidr: u8) -> u32 {
    1 << (32 - cidr)
}


impl Ipv4Network {

    pub const MAX_NETMASK: u32 = u32::MAX;

    /// Creates a new IPv4 Network
    pub fn new(a: u8, b: u8, c: u8, d: u8, cidr: u8) -> Result<Ipv4Network, Error> {
        let first = u32::from_be_bytes([a, b, c, d]);
        match Ipv4Network::is_valid(first, cidr) {
            true => Ok(Ipv4Network {first: first, cidr: cidr}),
            false => Err(Error::InvalidNetwork)
        }
    }

    pub fn hostcount(&self) -> u32 {
        cidr_to_hostcount(self.cidr)
    }

    pub fn subnets(&self, new_cidr: u8) -> NetworkIterator {
        NetworkIterator {
            current: self.first,
            stepping: cidr_to_hostcount(new_cidr),
            cidr: new_cidr,
            max: self.first + self.hostcount() - 1
        }
    }
    pub fn hosts(&self) -> HostIterator {
        HostIterator {
            current: self.first,
            max: self.first + self.hostcount()
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
    pub fn subnet(&self, other: &Self) -> bool {
        self.first() <= other.first() && other.last() <= self.last()
    }
    pub fn supernet(&self, other: &Self) -> bool {
        self.first() >= other.first() && other.last() >= self.last()
    }
    pub fn netmask(&self) -> Ipv4Addr {
        let numeric = Ipv4Network::MAX_NETMASK ^ (self.hostcount() -1);
        Ipv4Addr::from(numeric)
    }

    #[inline(always)]
    fn is_valid(first: u32, cidr: u8) -> bool {
        first % cidr_to_hostcount(cidr) == 0
    }
}

impl Iterator for NetworkIterator {
    type Item = Ipv4Network;
    fn next(&mut self) -> Option<Ipv4Network> {
        if self.current <  self.max {
            self.current += self.stepping;
            let bytes = self.current.to_be_bytes();
            match Ipv4Network::new(bytes[0], bytes[1], bytes[2], bytes[3], self.cidr) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current as usize, Some(self.max as usize))
    }
}
impl Ord for Ipv4Network {
    fn cmp(&self, other: &Self) -> Ordering {
        let order = self.first().cmp(&other.first());
        match order {
            Ordering::Equal => self.cidr.cmp(&other.cidr),
            _ => order
        }
    }
}

impl PartialOrd for Ipv4Network {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>{
        Some(self.cmp(other))
    }
}
impl PartialEq for Ipv4Network {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.cidr == other.cidr
    }
}
impl FromStr for Ipv4Network {
    type Err = Error;

    fn from_str(s: &str) -> Result<Ipv4Network, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            2 => {
                let ip_first: Ipv4Addr = match parts[0].parse() {
                    Ok(ip_addr) => ip_addr,
                    Err(_) => return Err(Self::Err::NetworkParseError)
                };
                let cidr: u8 = match parts[1].parse() {
                    Ok(cidr) => cidr,
                    Err(_) => return Err(Self::Err::NetworkParseError)
                };
                let ip_tuple = u32::from(ip_first).to_be_bytes();
                Ipv4Network::new(ip_tuple[0], ip_tuple[1], ip_tuple[2], ip_tuple[3], cidr)
            },
            _ => Err(Self::Err::NetworkParseError)
        }
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
            Ipv4Network::new(1, 1, 1, 0, 25)
        );
    }
    #[test]
    fn new_network_invalid() {
        assert_eq!(
            Err(Error::InvalidNetwork),
            Ipv4Network::new(1, 1, 1, 0, 23)
        );
    }
    #[test]
    fn first_address() {
        let network = Ipv4Network::new(1, 1, 1, 0, 24).unwrap();
        let first: Ipv4Addr = "1.1.1.0".parse().unwrap();
        assert_eq!(first, network.first());

    }
    #[test]
    fn last_address() {
        let network = Ipv4Network::new(1, 1, 1, 0, 24).unwrap();
        let last: Ipv4Addr = Ipv4Addr::from_str("1.1.1.255").unwrap();
        assert_eq!(last, network.last());
    }
    #[test]
    fn contains_addr() {
        let network = Ipv4Network::new(1, 1, 1, 0, 24).unwrap();
        assert_eq!(network.contains(&Ipv4Addr::new(1,1,1,1)), true);
    }
    #[test]
    fn iterate() {
        let network = Ipv4Network::new(1, 1, 1, 0, 24).unwrap();
        let test = network.subnets(25);
        assert_eq!(test.stepping, 128);
        let test2: Vec<Ipv4Network> = test.collect();
        assert_eq!(test2.len(), 2);
    }
    #[test]
    fn test_from_string() {
        let res = Ipv4Network::from_str("1.1.1.0/24");
        assert_eq!(Ok(Ipv4Network{first: 16843008, cidr: 24}), res)
    }
    #[test]
    fn test_from_string_fail() {
        let res = Ipv4Network::from_str("1.1.1.1");
        assert_eq!(Err(Error::NetworkParseError), res)
    }
    #[test]
    fn test_subnet() {
        let supernet = Ipv4Network::from_str("1.0.0.0/22").unwrap();
        let subnet = Ipv4Network::from_str("1.0.1.0/24").unwrap();
        assert_eq!(supernet.subnet(&subnet), true);
    }
    #[test]
    fn test_supernet() {
        let supernet = Ipv4Network::from_str("1.0.0.0/22").unwrap();
        let subnet = Ipv4Network::from_str("1.0.1.0/24").unwrap();
        assert_eq!(subnet.supernet(&supernet), true);
    }
    #[test]
    fn test_bigger() {
        let supernet = Ipv4Network::from_str("1.0.0.0/22").unwrap();
        let subnet = Ipv4Network::from_str("1.0.1.0/24").unwrap();
        assert_eq!(&subnet > &supernet, true);
    }
    #[test]
    fn test_parse() {
        let network = "1.1.1.0/24".parse();
        assert_eq!(Ok(Ipv4Network{first: 16843008, cidr: 24}), network)
    }
}
