use crate::gen::gnmi::GetResponse;

/// Get data from a given gNMI Target device.
///
/// Retrieves paths based on their data type from a gNMI Target device.
/// Obtained via [`Client::get`].
#[derive(Debug, Clone)]
pub struct Notifications(pub GetResponse);

impl<'a> Notifications {
}
