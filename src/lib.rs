//! A Rust client for the gRPC Network Management Interface
//!
//! [ginmi](https://github.com/Lachstec/ginmi) is a crate based on [tonic](https://github.com/hyperium/tonic) for communicating with network
//! devices that support [gNMI](https://openconfig.net/docs/gnmi/gnmi-specification/). It enables
//! querying and manipulating of the device configuration and status.
//! 
//! # Feature Flags
//! - `dangerous_configuration`: allows for insecure configurations, for example not validating TLS-Certificates
mod auth;
mod client;
mod error;

pub use client::{Capabilities, Client, ClientBuilder, Encoding};
pub use error::GinmiError;

#[cfg(feature = "dangerous_configuration")]
pub use client::{DangerousClientBuilder, DangerousConnection};

pub(crate) mod gen {
    pub mod gnmi {
        tonic::include_proto!("gnmi");
    }

    pub mod gnmi_ext {
        tonic::include_proto!("gnmi_ext");
    }

    pub mod target {
        tonic::include_proto!("target");
    }

    pub mod google {
        pub mod protobuf {
            tonic::include_proto!("google.protobuf");
        }
    }
}
