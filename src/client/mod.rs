mod capabilities;
mod client;
#[cfg(feature = "dangerous_configuration")]
#[cfg_attr(docsrs, doc(cfg(feature = "dangerous_configuration")))]
mod dangerous;

pub use client::{Client, ClientBuilder};

pub use capabilities::{Capabilities, Encoding};

#[cfg(feature = "dangerous_configuration")]
pub use dangerous::{DangerousClientBuilder, DangerousConnection};
