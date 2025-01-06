use serde::{Deserialize, Serialize};

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
    pub pressure_unit: Option<String>,
    pub pressure_value: Option<f64>,
    pub is_new_atis: Option<bool>,
    pub text_atis: Option<String>,
}
