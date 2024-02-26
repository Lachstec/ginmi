//! A Rust client for the gRPC Network Management Interface
//!
//! Provides a Client to modify and retrieve configuration from target network devices,
//! as well as various telemetry data.
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
