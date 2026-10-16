use crate::pty::WakeupCallback;
use serde::Serialize;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialParity {
    None,
    Even,
    Odd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialFlowControl {
    None,
    Software,
    Hardware,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: SerialParity,
    pub stop_bits: u8,
    pub flow_control: SerialFlowControl,
}
impl SerialConfig {
    pub fn new(
        baud_rate: u32,
        data_bits: u8,
        parity: SerialParity,
        stop_bits: u8,
        flow_control: SerialFlowControl,
    ) -> io::Result<Self> {
        if !(5..=8).contains(&data_bits) || !(1..=2).contains(&stop_bits) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid serial configuration",
            ));
        }
        Ok(Self {
            baud_rate,
            data_bits,
            parity,
            stop_bits,
            flow_control,
        })
    }
    pub fn summary(&self) -> String {
        format!("{} baud", self.baud_rate)
    }
}
#[derive(Serialize)]
pub struct SerialPortInfo {
    pub path: String,
    pub display_name: String,
    pub description: Option<String>,
}
pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    Vec::new()
}

pub struct SerialTransport;
pub struct SerialPump {
    pub output: Vec<u8>,
    pub exited: bool,
    pub error: Option<String>,
    pub has_more: bool,
}
impl SerialTransport {
    pub fn open(_: &str, _: SerialConfig) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "desktop serial ports are unavailable on Android",
        ))
    }
    pub fn path(&self) -> &str {
        ""
    }
    pub fn config(&self) -> SerialConfig {
        unreachable!("Android serial transport cannot be opened")
    }
    pub fn queue_input(&mut self, _: &[u8]) -> bool {
        false
    }
    pub fn set_wakeup_callback(&mut self, _: Option<WakeupCallback>) {}
    pub fn drain_output(&mut self) -> SerialPump {
        SerialPump {
            output: Vec::new(),
            exited: true,
            error: None,
            has_more: false,
        }
    }
    pub fn clear_pending_output(&mut self) {}
}
