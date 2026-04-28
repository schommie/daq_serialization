use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsFrame;

use ws_protocol::*;

const SERVER_ADDR: &str = "127.0.0.1:9002";

fn to_ws_frame(message: &ws_protocol::Message) -> WsFrame {
    let json = message
        .encode_json()
        .expect("Message should always serialize");
    WsFrame::text(json)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    println!("pi websocket server listening on ws://{SERVER_ADDR}");
    println!(
        r#"try sending: {{"system":"daq","message":{{"frame":"wheelSpeed","source":"nodefl","rpm":42.0}}}}"#
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

    let hello = ws_protocol::Message::Daq(DaqMessage::Temperature {
        source: Device::Raspi,
        samples: [TemperatureSample {
            tire: Celsius(23.5),
            brake: Celsius(24.0),
        }; TEMPERATURE_SAMPLE_COUNT],
    });

    //println!("sending hello:\n{}", hello.to_pretty_json());
    //ws_tx.send(to_ws_frame(&hello)).await?;

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(WsFrame::Text(text)) => {
                    //println!("device -> pi raw: {text}");

                    match ws_protocol::Message::decode_json(&text) {
                        Ok(ws_message) => {
                            /*
                            println!(
                                "device -> pi deserialized:\n{}",
                                ws_message.to_pretty_json()
                            );
                            */
                            handle_ws_message(&ws_message);
                        }
                        Err(_) => println!("device -> pi did not match Message"),
                    }
                }
                Ok(WsFrame::Close(_)) => break,
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

fn handle_ws_message(message: &ws_protocol::Message) {
    match message {
        ws_protocol::Message::Daq(message) => handle_daq_message(message),
        ws_protocol::Message::Bms(message) => handle_bms_message(message),
        ws_protocol::Message::Vcu(message) => handle_vcu_message(message),
    }
}

fn handle_daq_message(message: &DaqMessage) {
    match message {
        DaqMessage::Temperature { source, samples } => {
            println!("Temperature Samples from {source:?}:");
            for (i, sample) in samples.iter().enumerate() {
                println!("Temperature Sample {}: Brake temp - {} Tire Temp - {}", i, sample.brake.0, sample.tire.0);
            }
        }
        DaqMessage::WheelSpeed { source, rpm } => {
            println!("device -> pi daq wheel speed from {source:?}: {rpm:?}");
        }
        DaqMessage::Imu { source, samples } => {
            println!("device -> pi daq imu from {source:?}: {samples:?}");
        }
        DaqMessage::Tbd { source, value } => {
            println!("device -> pi daq tbd from {source:?}: {value:?}");
        }
    }
}

fn handle_bms_message(message: &BmsMessage) {
    match message {
        BmsMessage::Voltages { source, readings } => {
            println!("device -> pi bms voltages from {source:?}: {readings:?}");
        }
        BmsMessage::Temperatures { source, readings } => {
            println!("device -> pi bms temperatures from {source:?}: {readings:?}");
        }
        BmsMessage::Balancing {
            source,
            active_cell,
            duty_cycle,
        } => {
            println!(
                "device -> pi bms balancing from {source:?}: active cell {active_cell}, duty cycle {duty_cycle:?}"
            );
        }
        BmsMessage::Faults {
            source,
            code,
            severity,
        } => {
            println!("device -> pi bms fault from {source:?}: code {code}, severity {severity:?}");
        }
        BmsMessage::SetValue { source, target } => {
            println!("device -> pi bms set value from {source:?}: {target:?}");
        }
        BmsMessage::Reset { source } => {
            println!("device -> pi bms reset from {source:?}");
        }
        BmsMessage::Ping { source } => {
            println!("device -> pi bms ping from {source:?}");
        }
    }
}

fn handle_vcu_message(message: &VcuMessage) {
    match message {
        VcuMessage::TorqueRequest { source, torque } => {
            println!("device -> pi vcu torque request from {source:?}: {torque:?}");
        }
        VcuMessage::SetValue { source, target } => {
            println!("device -> pi vcu set value from {source:?}: {target:?}");
        }
        VcuMessage::Reset { source } => {
            println!("device -> pi vcu reset from {source:?}");
        }
        VcuMessage::Ping { source } => {
            println!("device -> pi vcu ping from {source:?}");
        }
    }
}
