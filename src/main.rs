use std::net::Ipv4Addr;

fn main() {
    let n = ipnetwork::Ipv4Network::new(16777216, 8).unwrap();
    let _networks: Vec<Ipv4Addr> = n.subnets(32).map(|n| n.first()).collect();

    println!("{}", _networks.get(0).unwrap());
}