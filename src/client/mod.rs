mod client;
mod capabilities;
#[cfg(feature = "dangerous_configuration")]
mod dangerous;

pub use client::{Client, ClientBuilder};

pub use capabilities::{
    Capabilities,
    Encoding
};