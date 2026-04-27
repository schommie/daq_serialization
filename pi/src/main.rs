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
pub enum SystemType {
    Daq,
    Bms,
    Vcu,
}

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
#[serde(rename_all = "camelCase")]
pub enum Frame {
    Voltages,
    Temperatures,
    Balancing,
    Faults,
    Temperature,
    WheelSpeed,
    Imu,
    TorqueRequest,
    #[serde(rename = "tbd")]
    Tbd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    SetValue,
    Reset,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    Telemetry {
        system: SystemType,
        frame: Frame,
        device: Device,
        values: HashMap<String, f32>,
    },
    Command {
        system: SystemType,
        command: Command,
        device: Device,
        values: HashMap<String, f32>,
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
        r#"try sending: {{"type":"telemetry","system":"daq","frame":"temperature","device":"nodefl","values":{{"rpm":42.0}}}}"#
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
    let hello = WsMessage::Telemetry {
        system: SystemType::Daq,
        frame: Frame::Temperature,
        device: Device::Raspi,
        values: hello_data,
    };

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
        WsMessage::Telemetry {
            system,
            frame,
            device,
            values,
        } => {
            println!("device -> pi telemetry from {device:?} ({system:?} {frame:?}): {values:?}");
        }
        WsMessage::Command {
            system,
            command,
            device,
            values,
        } => {
            println!("device -> pi command from {device:?} ({system:?} {command:?}): {values:?}");
        }
    }
}
