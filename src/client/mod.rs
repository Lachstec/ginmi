use crate::gen::gnmi::g_nmi_client::*;
use crate::error::GinmiError;
use tonic::transport::Channel;

type ClientConn = GNmiClient<Channel>;

#[derive(Debug, Clone)]
pub struct Client(ClientConn);

impl Client {
    pub async fn new(url: &str) -> Result<Self, GinmiError> {
        let client = GNmiClient::connect(String::from(url)).await?;
        Ok(Self(client))
    }
}