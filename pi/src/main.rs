use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

const SERVER_ADDR: &str = "127.0.0.1:9002";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Bms,
    Vcu,
    Raspi,
    NodeFL,
    NodeFR,
    NodeRL,
    NodeRR,
    NodeDash,
    NodeRideHeight,
    NodePDMTB,
    NodePDMDASH,
    NodePDMPCBPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "system", content = "message", rename_all = "lowercase")]
pub enum WsMessage {
    Daq(DaqMessage),
    Bms(BmsMessage),
    Vcu(VcuMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum DaqMessage {
    Temperature {
        device: Device,
        values: HashMap<String, f32>,
    },
    WheelSpeed {
        device: Device,
        values: HashMap<String, f32>,
    },
    Imu {
        device: Device,
        values: HashMap<String, f32>,
    },
    #[serde(rename = "tbd")]
    Tbd {
        device: Device,
        values: HashMap<String, f32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum BmsMessage {
    Voltages {
        device: Device,
        values: HashMap<String, f32>,
    },
    Temperatures {
        device: Device,
        values: HashMap<String, f32>,
    },
    Balancing {
        device: Device,
        values: HashMap<String, f32>,
    },
    Faults {
        device: Device,
        values: HashMap<String, f32>,
    },
    SetValue {
        device: Device,
        values: HashMap<String, f32>,
    },
    Reset {
        device: Device,
    },
    Ping {
        device: Device,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum VcuMessage {
    TorqueRequest {
        device: Device,
        values: HashMap<String, f32>,
    },
    SetValue {
        device: Device,
        values: HashMap<String, f32>,
    },
    Reset {
        device: Device,
    },
    Ping {
        device: Device,
    },
}

impl WsMessage {
    pub fn to_ws_message(&self) -> Message {
        let json = serde_json::to_string(self).expect("WsMessage should always serialize");
        Message::text(json)
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("WsMessage should always serialize")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    println!("pi websocket server listening on ws://{SERVER_ADDR}");
    println!(
        r#"try sending: {{"system":"daq","message":{{"frame":"temperature","device":"nodefl","values":{{"rpm":42.0}}}}}}"#
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("client connected: {addr}");

        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, addr).await {
                eprintln!("client {addr} error: {error}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let ws = accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    let hello_data = HashMap::from([("connected".to_string(), 1.0)]);
    let hello = WsMessage::Daq(DaqMessage::Temperature {
        device: Device::Raspi,
        values: hello_data,
    });

    println!("sending hello:\n{}", hello.to_pretty_json());
    ws_tx.send(hello.to_ws_message()).await?;

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    println!("device -> pi raw: {text}");

                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_message) => {
                            println!(
                                "device -> pi deserialized:\n{}",
                                ws_message.to_pretty_json()
                            );
                            print_ws_message_summary(&ws_message);
                        }
                        Err(_) => println!("device -> pi did not match WsMessage"),
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(error) => {
                    eprintln!("client {addr} rx error: {error}");
                    break;
                }
            }
        }
    });

    let _ = rx_thread.await;

    println!("client disconnected: {addr}");
    Ok(())
}

fn print_ws_message_summary(message: &WsMessage) {
    match message {
        WsMessage::Daq(message) => handle_daq_message(message),
        WsMessage::Bms(message) => handle_bms_message(message),
        WsMessage::Vcu(message) => handle_vcu_message(message),
    }
}

fn handle_daq_message(message: &DaqMessage) {
    match message {
        DaqMessage::Temperature { device, values } => {
            println!("device -> pi daq temperature from {device:?}: {values:?}");
        }
        DaqMessage::WheelSpeed { device, values } => {
            println!("device -> pi daq wheel speed from {device:?}: {values:?}");
        }
        DaqMessage::Imu { device, values } => {
            println!("device -> pi daq imu from {device:?}: {values:?}");
        }
        DaqMessage::Tbd { device, values } => {
            println!("device -> pi daq tbd from {device:?}: {values:?}");
        }
    }
}

fn handle_bms_message(message: &BmsMessage) {
    match message {
        BmsMessage::Voltages { device, values } => {
            println!("device -> pi bms voltages from {device:?}: {values:?}");
        }
        BmsMessage::Temperatures { device, values } => {
            println!("device -> pi bms temperatures from {device:?}: {values:?}");
        }
        BmsMessage::Balancing { device, values } => {
            println!("device -> pi bms balancing from {device:?}: {values:?}");
        }
        BmsMessage::Faults { device, values } => {
            println!("device -> pi bms faults from {device:?}: {values:?}");
        }
        BmsMessage::SetValue { device, values } => {
            println!("device -> pi bms set value from {device:?}: {values:?}");
        }
        BmsMessage::Reset { device } => {
            println!("device -> pi bms reset from {device:?}");
        }
        BmsMessage::Ping { device } => {
            println!("device -> pi bms ping from {device:?}");
        }
    }
}

fn handle_vcu_message(message: &VcuMessage) {
    match message {
        VcuMessage::TorqueRequest { device, values } => {
            println!("device -> pi vcu torque request from {device:?}: {values:?}");
        }
        VcuMessage::SetValue { device, values } => {
            println!("device -> pi vcu set value from {device:?}: {values:?}");
        }
        VcuMessage::Reset { device } => {
            println!("device -> pi vcu reset from {device:?}");
        }
        VcuMessage::Ping { device } => {
            println!("device -> pi vcu ping from {device:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_daq_message_with_system_router() {
        let message = WsMessage::Daq(DaqMessage::Temperature {
            device: Device::NodeFL,
            values: HashMap::from([("celsius".to_string(), 23.5)]),
        });

        let json = serde_json::to_value(&message).expect("message should serialize");

        assert_eq!(json["system"], "daq");
        assert_eq!(json["message"]["frame"], "temperature");
        assert_eq!(json["message"]["device"], "nodefl");
        assert_eq!(json["message"]["values"]["celsius"], 23.5);
    }

    #[test]
    fn deserializes_bms_message_without_command_or_telemetry_type() {
        let json = r#"{
            "system": "bms",
            "message": {
                "frame": "setValue",
                "device": "bms",
                "values": { "target": 12.0 }
            }
        }"#;

        let message: WsMessage = serde_json::from_str(json).expect("message should deserialize");

        match message {
            WsMessage::Bms(BmsMessage::SetValue { device, values }) => {
                assert!(matches!(device, Device::Bms));
                assert_eq!(values["target"], 12.0);
            }
            other => panic!("expected BMS setValue message, got {other:?}"),
        }
    }
}
