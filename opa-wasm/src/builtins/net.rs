use std::net::IpAddr;

use ipnetwork::IpNetwork;

use crate::value::Set;
use crate::{Error, Value};

enum AddrOrNetwork {
    Addr(IpAddr),
    Network(IpNetwork),
}

pub fn cidr_contains(cidr: Value, cidr_or_ip: Value) -> Result<Value, Error> {
    let cidr = cidr
        .try_into_string()?
        .parse::<IpNetwork>()
        .map_err(Error::InvalidIpNetwork)?;
    let cidr_or_ip = cidr_or_ip.try_into_string()?;
    let cidr_or_ip = cidr_or_ip
        .parse::<IpAddr>()
        .map(AddrOrNetwork::Addr)
        .or_else(|_| cidr_or_ip.parse::<IpNetwork>().map(AddrOrNetwork::Network))
        .map_err(Error::InvalidIpNetwork)?;
    let v = match (cidr, cidr_or_ip) {
        (cidr, AddrOrNetwork::Addr(addr)) => cidr.contains(addr),
        (IpNetwork::V4(cidr), AddrOrNetwork::Network(IpNetwork::V4(network))) => {
            cidr.is_supernet_of(network)
        }
        (IpNetwork::V6(cidr), AddrOrNetwork::Network(IpNetwork::V6(network))) => {
            cidr.is_supernet_of(network)
        }
        _ => false,
    };
    Ok(v.into())
}

pub fn cidr_intersects(cidr1: Value, cidr2: Value) -> Result<Value, Error> {
    let cidr1 = cidr1
        .try_into_string()?
        .parse::<IpNetwork>()
        .map_err(Error::InvalidIpNetwork)?;
    let cidr2 = cidr2
        .try_into_string()?
        .parse::<IpNetwork>()
        .map_err(Error::InvalidIpNetwork)?;
    let v = match (cidr1, cidr2) {
        (IpNetwork::V4(cidr1), IpNetwork::V4(cidr2)) => cidr1.overlaps(cidr2),
        (IpNetwork::V6(cidr1), IpNetwork::V6(cidr2)) => cidr1.overlaps(cidr2),
        _ => false,
    };
    Ok(v.into())
}

pub fn cidr_expand(cidr: Value) -> Result<Value, Error> {
    let cidr = cidr
        .try_into_string()?
        .parse::<IpNetwork>()
        .map_err(Error::InvalidIpNetwork)?;
    let v = cidr
        .iter()
        .map(|a| a.to_string())
        .map(Into::into)
        .collect::<Set<Value>>();
    Ok(v.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_cidr_contains() {
        let cidr = "127.0.0.1/16".into();
        let ip = "127.0.0.2".into();
        assert_eq!(
            true,
            cidr_contains(cidr, ip).unwrap().try_into_bool().unwrap()
        );

        let cidr = "127.0.0.1/16".into();
        let net = "127.0.0.1/16".into();
        assert_eq!(
            true,
            cidr_contains(cidr, net).unwrap().try_into_bool().unwrap()
        );

        let cidr = "127.0.0.1/16".into();
        let ip = "172.18.0.1".into();
        assert_eq!(
            false,
            cidr_contains(cidr, ip).unwrap().try_into_bool().unwrap()
        );

        let cidr = "127.0.0.1/16".into();
        let net = "127.0.0.1/15".into();
        assert_eq!(
            false,
            cidr_contains(cidr, net).unwrap().try_into_bool().unwrap()
        );
    }

    #[test]
    fn test_net_cidr_intersects() {
        let cidr1 = "192.168.0.0/16".into();
        let cidr2 = "192.168.1.0/24".into();
        assert_eq!(
            true,
            cidr_intersects(cidr1, cidr2)
                .unwrap()
                .try_into_bool()
                .unwrap()
        );
    }
}
