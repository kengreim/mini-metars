use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtisUpdateMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub value: AtisUpdateValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Observer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum AtisType {
    Combined,
    Departure,
    Arrival,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueUnit {
    pub actual_value: f64,
    pub actual_unit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtisUpdateValue {
    pub network_connection_status: Option<NetworkConnectionStatus>,
    pub station: Option<String>,
    pub atis_type: Option<AtisType>,
    pub atis_letter: Option<String>,
    pub metar: Option<String>,
    pub wind: Option<String>,
    pub altimeter: Option<String>,
    pub pressure: Option<ValueUnit>,
    pub ceiling: Option<ValueUnit>,
    pub prevailing_visibility: Option<ValueUnit>,
    pub is_new_atis: Option<bool>,
    pub text_atis: Option<String>,
}

impl AtisUpdateValue {
    pub fn letter_or(&self, str: &str) -> String {
        self.atis_letter
            .as_ref()
            .map_or_else(|| str.to_string(), std::string::ToString::to_string)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtisUpdateRequest {
    #[serde(rename = "type")]
    msg_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<AtisUpdateStation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtisUpdateStation {
    pub station: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atis_type: Option<AtisType>,
}

impl AtisUpdateRequest {
    pub const fn new_all() -> Self {
        Self {
            msg_type: "getAtis",
            value: None,
        }
    }
}

impl TryFrom<AtisUpdateRequest> for Message {
    type Error = serde_json::Error;

    fn try_from(value: AtisUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self::text(serde_json::to_string(&value)?))
    }
}
