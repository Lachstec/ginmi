use crate::gen::gnmi::CapabilityResponse;
use crate::gen::gnmi::ModelData;


pub use crate::gen::gnmi::Encoding;

/// Capabilities of a given gNMI Target device.
///
/// Contains information about the capabilities that supported by a gNMI Target device.
#[derive(Debug, Clone)]
pub struct Capabilities(pub(crate) CapabilityResponse);

impl<'a> Capabilities {
    /// Retrieve the gNMI Version that the target device supports.
    pub fn gnmi_version(&'a self) -> &'a str {
        self.0.g_nmi_version.as_str()
    }

    /// Check if target device supports a given model.
    ///
    /// # Arguments
    /// - name: Name of the model
    /// - organization: Organization publishing the model
    /// - version: Version of the model
    pub fn supports_model(&self, name: &str, organization: &str, version: &str) -> bool {
        self.0.supported_models.contains(&ModelData {
            name: name.to_string(),
            organization: organization.to_string(),
            version: version.to_string()
        })
    }

    /// Check if a target device supports a given [`Encoding`].
    ///
    /// # Arguments
    /// - encoding: The [`Encoding`] to check for.
    pub fn supports_encoding(&self, encoding: Encoding) -> bool {
        let enc: i32 = match encoding {
            Encoding::Json => 0,
            Encoding::Bytes => 1,
            Encoding::Proto => 2,
            Encoding::Ascii => 3,
            Encoding::JsonIetf => 4
        };

        self.0.supported_encodings.contains(&enc)
    }
}

