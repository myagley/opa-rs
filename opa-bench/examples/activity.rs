use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use criterion::black_box;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientId(String);

impl From<String> for ClientId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Authenticated MQTT client identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthId {
    /// Identity for anonymous client.
    Anonymous,

    /// Identity for non-anonymous client.
    Identity(Identity),
}

impl std::fmt::Display for AuthId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anonymous => write!(f, "*"),
            Self::Identity(identity) => write!(f, "{}", identity),
        }
    }
}

impl AuthId {
    /// Creates a MQTT identity for known client.
    pub fn from_identity<T: Into<Identity>>(identity: T) -> Self {
        Self::Identity(identity.into())
    }
}

impl<T: Into<Identity>> From<T> for AuthId {
    fn from(identity: T) -> Self {
        AuthId::from_identity(identity)
    }
}

/// Non-anonymous client identity.
pub type Identity = String;

/// Describes a client activity to authorized.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Activity {
    auth_id: AuthId,
    client_id: ClientId,
    operation: Operation,
}

impl Activity {
    pub fn new(
        auth_id: impl Into<AuthId>,
        client_id: impl Into<ClientId>,
        operation: Operation,
    ) -> Self {
        Self {
            auth_id: auth_id.into(),
            client_id: client_id.into(),
            operation,
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn auth_id(&self) -> &AuthId {
        &self.auth_id
    }

    pub fn operation(&self) -> &Operation {
        &self.operation
    }
}

/// Describes a client operation to be authorized.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Connect(Connect),
    Publish(Publish),
    Subscribe(Subscribe),
    Receive(Receive),
}

impl Operation {
    /// Creates a new operation context for CONNECT request.
    pub fn new_connect() -> Self {
        let c = Connect {
            remote_addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            will: None,
        };
        Self::Connect(c)
    }

    // /// Creates a new operation context for PUBLISH request.
    // pub fn new_publish(publish: proto::Publish) -> Self {
    //     Self::Publish(publish.into())
    // }
    //
    // /// Creates a new operation context for SUBSCRIBE request.
    // pub fn new_subscribe(subscribe_to: proto::SubscribeTo) -> Self {
    //     Self::Subscribe(subscribe_to.into())
    // }
    //
    // /// Creates a new operation context for RECEIVE request.
    // ///
    // /// RECEIVE request happens when broker decides to publish a message to a certain
    // /// topic client subscribed to.
    // pub fn new_receive(publication: proto::Publication) -> Self {
    //     Self::Receive(publication.into())
    // }
}

/// Represents a client attempt to connect to the broker.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Connect {
    remote_addr: IpAddr,
    will: Option<Publication>,
}

/// Represents a publication description without payload to be used for authorization.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Publication {
    // topic_name: String,
// qos: proto::QoS,
// retain: bool,
}

// impl Publication {
//     pub fn topic_name(&self) -> &str {
//         &self.topic_name
//     }
// }
//
// impl From<proto::Publication> for Publication {
//     fn from(publication: proto::Publication) -> Self {
//         Self {
//             topic_name: publication.topic_name,
//             qos: publication.qos,
//             retain: publication.retain,
//         }
//     }
// }

/// Represents a client attempt to publish a new message on a specified MQTT topic.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Publish {
    // publication: Publication,
}

// impl Publish {
//     pub fn publication(&self) -> &Publication {
//         &self.publication
//     }
// }
//
// impl From<proto::Publish> for Publish {
//     fn from(publish: proto::Publish) -> Self {
//         Self {
//             publication: Publication {
//                 topic_name: publish.topic_name,
//                 qos: match publish.packet_identifier_dup_qos {
//                     proto::PacketIdentifierDupQoS::AtMostOnce => proto::QoS::AtMostOnce,
//                     proto::PacketIdentifierDupQoS::AtLeastOnce(_, _) => proto::QoS::AtLeastOnce,
//                     proto::PacketIdentifierDupQoS::ExactlyOnce(_, _) => proto::QoS::ExactlyOnce,
//                 },
//                 retain: publish.retain,
//             },
//         }
//     }
// }

/// Represents a client attempt to subscribe to a specified MQTT topic in order to received messages.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Subscribe {
    // topic_filter: String,
// qos: proto::QoS,
}

// impl Subscribe {
//     pub fn topic_filter(&self) -> &str {
//         &self.topic_filter
//     }
// }
//
// impl From<proto::SubscribeTo> for Subscribe {
//     fn from(subscribe_to: proto::SubscribeTo) -> Self {
//         Self {
//             topic_filter: subscribe_to.topic_filter,
//             qos: subscribe_to.qos,
//         }
//     }
// }

/// Represents a client to received a message from a specified MQTT topic.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Receive {
    // publication: Publication,
}

// impl From<proto::Publication> for Receive {
//     fn from(publication: proto::Publication) -> Self {
//         Self {
//             publication: publication.into(),
//         }
//     }
// }

fn main() {
    let query = "data.test.allow";
    let mut module_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    module_path.push("examples/activity.rego");
    let module = std::fs::read_to_string(&module_path).unwrap();

    let mut rego = opa_rego::Policy::from_query(query, &[module.as_str()]).unwrap();

    for _i in 0..100_000_000 {
        let operation = Operation::new_connect();
        let activity = Activity::new(
            "auth_id".to_string(),
            ClientId("client_id".to_string()),
            operation,
        );
        let result: bool = rego.evaluate(&activity).unwrap();
        assert_eq!(true, result);
    }
}
