//! Provides a Client that can connect to gNMI-capable devices.
//!
//! The Client should be created by using the appropriate builder.
//! Clients can be reused and are cheap to clone.
//!
//! # Examples
//! Connecting to a Nokia SR-Linux Device running in [Containerlab](https://containerlab.dev/):
//! ```rust
//! # use ginmi::client::Client;
//! fn main() -> std::io::Result<()> {
//! # tokio_test::block_on(async {
//! # const CA_CERT: &str = "CA Certificate";
//! let mut client = Client::builder("https://clab-srl01-srl:57400")
//!     .tls(CA_CERT, "clab-srl01-srl")
//!     .credentials("admin", "password1")
//!     .build()
//!     .await?;
//! # })}
//! ```
mod capabilities;
mod client;
#[cfg(feature = "dangerous_configuration")]
#[cfg_attr(docsrs, doc(cfg(feature = "dangerous_configuration")))]
pub mod dangerous;

pub use client::{Client, ClientBuilder};

pub use capabilities::{Capabilities, Encoding};
