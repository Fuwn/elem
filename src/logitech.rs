// This file is part of elem <https://github.com/Fuwn/elem>.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
//
// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use tungstenite::{client::IntoClientRequest, Message};

use crate::tray::quit;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
  pub id: String,
  #[serde(rename = "connectionType")]
  connection_type: String,
  #[serde(rename = "deviceType")]
  device_type: String,
  #[serde(rename = "displayName")]
  pub display_name: String,
}

impl DeviceInfo {
  pub fn new(
    id: &str,
    connection_type: &str,
    device_type: &str,
    display_name: &str,
  ) -> Self {
    Self {
      id: id.to_string(),
      connection_type: connection_type.to_string(),
      device_type: device_type.to_string(),
      display_name: display_name.to_string(),
    }
  }

  pub fn from_device_info(device_info: &Self) -> Self {
    Self {
      id: device_info.id.clone(),
      connection_type: device_info.connection_type.clone(),
      device_type: device_info.device_type.clone(),
      display_name: device_info.display_name.clone(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceListPayload {
  #[serde(rename = "deviceInfos")]
  device_infos: Vec<DeviceInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceList {
  payload: DeviceListPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevicePayload {
  percentage: u64,
}

impl DevicePayload {
  pub const fn percentage(&self) -> u64 { self.percentage }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
  payload: DevicePayload,
}

impl Device {
  pub const fn payload(&self) -> &DevicePayload { &self.payload }
}

/// Create a connection to the Logitech G HUB `WebSocket` (backtick-ed because
/// rustfmt is forcing me to)
fn connection() -> tungstenite::WebSocket<
  tungstenite::stream::MaybeTlsStream<std::net::TcpStream>,
> {
  // This will never fail because the URL is hardcoded
  let url = url::Url::parse("ws://localhost:9010").unwrap();

  let (mut ws_stream, _) = tungstenite::connect({
    // This will never fail because the URL is valid
    let mut request = url.into_client_request().unwrap();

    // https://github.com/snapview/tungstenite-rs/issues/279
    // https://github.com/snapview/tungstenite-rs/issues/145#issuecomment-713581499
    //
    // This unwrap will never fail either because we are parsing a hardcoded,
    // known string
    request
      .headers_mut()
      .insert("Sec-WebSocket-Protocol", "json".parse().unwrap());

    request
  })
  .unwrap_or_else(|_| {
    quit("failed to connect to the logitech g hub websocket. is it running?");
  });

  ws_stream.read_message().unwrap_or_else(|_| {
    quit("failed to read message from the logitech g hub websocket");
  });

  ws_stream
}

/// Get a list of only wireless devices from the Logitech G HUB `WebSocket`
pub fn wireless_devices() -> HashMap<String, DeviceInfo> {
  let mut stream = connection();

  stream
    .write_message(Message::binary(
      serde_json::json!({
        "path": "/devices/list",
        "verb": "GET"
      })
      .to_string(),
    ))
    .unwrap_or_else(|_| {
      quit("failed to write message to the logitech g hub websocket");
    });

  // This will never fail because if we even got this far, the `WebSocket` is
  // working.
  let devices = serde_json::from_value::<DeviceList>(
    serde_json::from_str(&stream.read_message().unwrap().into_text().unwrap())
      .unwrap(),
  )
  .unwrap();
  let wireless = devices
    .payload
    .device_infos
    .iter()
    .filter(|device_info| device_info.connection_type == "WIRELESS")
    .map(DeviceInfo::from_device_info)
    .collect::<Vec<DeviceInfo>>();
  let mut mapped = HashMap::new();

  for device in wireless {
    mapped.insert(device.display_name.clone(), device);
  }

  // Adding a dummy device to the device list for testing purposes.
  //
  // I'm also going to keep this in because it's a nice way for the user to make
  // sure everything is working properly.
  mapped.insert(
    "Dummy (Debug)".to_string(),
    DeviceInfo::new("dummy_debug", "WIRELESS", "MOUSE", "Dummy (Debug)"),
  );

  mapped
}

/// Get the battery percentage of a specific wireless device
pub fn device(display_name: &str) -> Device {
  if display_name == "Dummy (Debug)" {
    return Device {
      payload: DevicePayload { percentage: 100 },
    };
  }

  let mut stream = connection();

  stream
    .write_message(Message::binary(
      // The `unwrap` for `get` should never fail because we know the devices that
      // are available, and if the user unplugs one of their devices mid-battery state check,
      // that's on them. :P
      serde_json::json!({
        "path": format!("/battery/{}/state", wireless_devices().get(display_name).unwrap().id),
        "verb": "GET"
      })
      .to_string(),
    ))
    .unwrap_or_else(|_| {
      quit("failed to write message to the logitech g hub websocket");
    });

  // Once again, this should never fail because if we even got this far, the
  // `WebSocket` is working.
  serde_json::from_value::<Device>(
    serde_json::from_str(&stream.read_message().unwrap().into_text().unwrap())
      .unwrap(),
  )
  .unwrap()
}
