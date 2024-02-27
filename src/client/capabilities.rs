use crate::gen::gnmi::CapabilityResponse;
use crate::gen::gnmi::ModelData;

pub use crate::gen::gnmi::Encoding;

/// Capabilities of a given gNMI Target device.
///
/// Contains information about the capabilities that supported by a gNMI Target device.
/// Obtained via [Client::capabilities](super::Client::capabilities).
#[derive(Debug, Clone)]
pub struct Capabilities(pub CapabilityResponse);

impl<'a> Capabilities {
    /// Retrieve the gNMI Version that the target device supports.
    ///
    /// # Examples
    /// ```rust
    /// # use ginmi::client::{Client, Capabilities};
    /// # fn main() -> std::io::Result<()> {
    /// # tokio_test::block_on(async {
    /// # const CERT: &str = "CA Certificate";
    /// let mut client = Client::builder("https://clab-srl01-srl:57400")
    ///     .tls(CERT, "clab-srl01-srl")
    ///     .credentials("admin", "admin")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let capabilities = client.capabilities().await.unwrap();
    /// let version = capabilities.gnmi_version();
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    pub fn gnmi_version(&'a self) -> &'a str {
        self.0.g_nmi_version.as_str()
    }

    /// Check if target device supports a given model.
    ///
    /// # Arguments
    /// - name: Name of the model
    /// - organization: Organization publishing the model
    /// - version: Version of the model
    ///
    /// # Examples
    /// ```rust
    /// # use ginmi::client::{Client, Capabilities};
    /// # fn main() -> std::io::Result<()> {
    /// # tokio_test::block_on(async {
    /// # const CERT: &str = "CA Certificate";
    /// let mut client = Client::builder("https://clab-srl01-srl:57400")
    ///     .tls(CERT, "clab-srl01-srl")
    ///     .credentials("admin", "admin")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let capabilities = client.capabilities().await.unwrap();
    /// let supports_aaa_nokia = capabilities.supports_model(
    ///     "urn:srl_nokia/aaa:srl_nokia-aaa",
    ///     "Nokia",
    ///     "2023-10-31"
    /// );
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    pub fn supports_model(&self, name: &str, organization: &str, version: &str) -> bool {
        self.0.supported_models.contains(&ModelData {
            name: name.to_string(),
            organization: organization.to_string(),
            version: version.to_string(),
        })
    }

    /// Check if a target device supports a given [`Encoding`].
    ///
    /// # Arguments
    /// - encoding: The [`Encoding`] to check for.
    ///
    /// # Examples
    /// ```rust
    /// # use ginmi::client::{Client, Capabilities, Encoding};
    /// # fn main() -> std::io::Result<()> {
    /// # tokio_test::block_on(async {
    /// # const CERT: &str = "CA Certificate";
    /// let mut client = Client::builder("https://clab-srl01-srl:57400")
    ///     .tls(CERT, "clab-srl01-srl")
    ///     .credentials("admin", "admin")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let capabilities = client.capabilities().await.unwrap();
    /// let supports_json = capabilities.supports_encoding(Encoding::Json);
    ///
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    pub fn supports_encoding(&self, encoding: Encoding) -> bool {
        let enc: i32 = match encoding {
            Encoding::Json => 0,
            Encoding::Bytes => 1,
            Encoding::Proto => 2,
            Encoding::Ascii => 3,
            Encoding::JsonIetf => 4,
        };

        self.0.supported_encodings.contains(&enc)
    }
}
