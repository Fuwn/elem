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

use std::sync::{Arc, Mutex};

use tao::{
  event::Event, event_loop::ControlFlow, menu, menu::CustomMenuItem,
  system_tray, system_tray::Icon,
};

const DEFAULT_UPDATE_FREQUENCY: u64 = 60000;

struct TrayInner {
  devices: Vec<CustomMenuItem>,
  selected_device_display_name: Option<String>,
  update_frequency: u64,
}

pub struct Tray {
  inner: Arc<Mutex<TrayInner>>,
}

impl Tray {
  pub fn new(update_frequency: Option<String>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(TrayInner {
        devices: vec![],
        selected_device_display_name: None,
        update_frequency: {
          update_frequency.map_or_else(
            || {
              debug!(
                "using default update frequency of {}ms",
                DEFAULT_UPDATE_FREQUENCY
              );

              DEFAULT_UPDATE_FREQUENCY
            },
            |update_frequency| match update_frequency.parse() {
              Ok(update_frequency) => {
                debug!(
                  "using custom update frequency of {}ms",
                  update_frequency
                );

                update_frequency
              }
              Err(e) => {
                warn!(
                  "invalid update frequency, using default of {}ms: {}",
                  DEFAULT_UPDATE_FREQUENCY, e
                );

                DEFAULT_UPDATE_FREQUENCY
              }
            },
          )
        },
      })),
    }
  }

  /// Force an icon by bypassing the device state check.
  ///
  /// Useful for displaying informational icons
  fn force_icon(code: &str) -> Icon {
    trace!("building forced icon: {}", code);

    let image = image::load_from_memory(&crate::ascii_art::number_to_image(
      code.parse().unwrap(),
    ))
    .unwrap()
    .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height).unwrap();

    trace!("built forced icon: {}", code);

    icon
  }

  /// Create a tray icon compatible icon from a devices battery level
  fn icon(selected_device_display_name: &Option<String>) -> Icon {
    trace!(
      "building icon for display name: {:?}",
      selected_device_display_name
    );

    let image = image::load_from_memory(&if selected_device_display_name
      == &Some("43770".to_string())
      || selected_device_display_name == &Some("Dummy (Debug)".to_string())
    {
      crate::ascii_art::number_to_image(43770)
    } else {
      crate::ascii_art::number_to_image(
        crate::logitech::device(
          &selected_device_display_name
            .clone()
            .unwrap_or_else(|| "1337".to_string()),
        )
        .payload()
        .percentage(),
      )
    })
    .unwrap()
    .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height).unwrap();

    trace!(
      "built icon for display name: {:?}",
      selected_device_display_name
    );

    icon
  }

  /// Checks and update the battery level of non-dummy devices
  fn watchman(
    icon_self: &Arc<Mutex<TrayInner>>,
    system_tray_updater: &Arc<Mutex<system_tray::SystemTray>>,
  ) {
    loop {
      std::thread::sleep(std::time::Duration::from_millis(
        icon_self.lock().unwrap().update_frequency,
      ));

      trace!("checking for system tray icon update");

      // Only refresh the tray icon (battery level) if the device is not a dummy
      // device
      if icon_self.lock().unwrap().selected_device_display_name
        != Some("Dummy (Debug)".to_string())
      {
        // "80085" is the internal code for ellipsis. An ellipsis is displayed
        // while the battery level is being fetched.
        system_tray_updater
          .lock()
          .unwrap()
          .set_icon(Self::force_icon("80085"));
        system_tray_updater.lock().unwrap().set_tooltip(&format!(
          "elem (updating {} from watchman)",
          &icon_self
            .lock()
            .unwrap()
            .selected_device_display_name
            .clone()
            .unwrap_or_else(|| "Dummy (Debug)".to_string())
        ));

        trace!("updating system tray icon from watchman");

        let icon = Self::icon(&Some(
          icon_self
            .lock()
            .unwrap()
            .selected_device_display_name
            .clone()
            .unwrap_or_else(|| "Dummy (Debug)".to_string()),
        ));

        system_tray_updater.lock().unwrap().set_tooltip(&format!(
          "elem ({})",
          icon_self
            .lock()
            .unwrap()
            .selected_device_display_name
            .clone()
            .unwrap_or_else(|| "Dummy (Debug)".to_string()),
        ));
        system_tray_updater.lock().unwrap().set_icon(icon);
        trace!("updated system tray icon",);
      }
    }
  }

  /// Run the tray icon and event loop
  #[allow(clippy::too_many_lines)]
  pub fn run(&mut self) {
    let local_self = self.inner.clone();
    // Grab all wireless devices
    let devices = crate::logitech::wireless_devices();
    // Set up the event loop and tray icon-related stuff
    let event_loop = tao::event_loop::EventLoop::new();
    let main_tray_id = tao::TrayId::new("main-tray");
    let mut tray_menu = menu::ContextMenu::new();

    tray_menu.add_item(
      menu::MenuItemAttributes::new(&format!(
        "Update frequency: {}ms",
        local_self.lock().unwrap().update_frequency
      ))
      .with_enabled(false),
    );

    // Adding all wireless devices to the tray icons devices menu
    tray_menu.add_submenu("Devices", true, {
      let mut menu = menu::ContextMenu::new();
      let mut devices = devices
        .values()
        .collect::<Vec<&crate::logitech::DeviceInfo>>();

      // Making sure that the last device, the default device, is never the
      // dummy device
      if devices.last().unwrap().display_name == "Dummy (Debug)" {
        let last = devices.pop().unwrap();

        devices.insert(0, last);
      }

      local_self.lock().unwrap().devices.clear();

      for (i, device_info) in devices.iter().enumerate() {
        let mut id = menu
          .add_item(menu::MenuItemAttributes::new(&device_info.display_name));

        if i == devices.len() - 1 {
          id.set_selected(true);

          local_self.lock().unwrap().selected_device_display_name =
            Some(device_info.display_name.to_string());
        }

        local_self.lock().unwrap().devices.push(id);
      }

      menu
    });

    let quit = tray_menu.add_item(menu::MenuItemAttributes::new("Quit"));
    let system_tray = Arc::new(Mutex::new(
      system_tray::SystemTrayBuilder::new(
        Self::icon(&local_self.lock().unwrap().selected_device_display_name),
        Some(tray_menu),
      )
      .with_id(main_tray_id)
      .with_tooltip("elem")
      .build(&event_loop)
      .unwrap(),
    ));
    let mut devices = local_self.lock().unwrap().devices.clone();
    let icon_self = self.inner.clone();
    let system_tray_updater = system_tray.clone();

    // An thread which updates the tray icon (battery level) every minute
    std::thread::spawn(move || {
      Self::watchman(&icon_self, &system_tray_updater);
    });

    // The event loop which takes care of switching devices, handling menu
    // events, and updating the device icon (battery level)
    event_loop.run(move |event, _event_loop, control_flow| {
      *control_flow = ControlFlow::Wait;

      if let Event::MenuEvent {
        menu_id,
        origin: menu::MenuType::ContextMenu,
        ..
      } = event
      {
        if menu_id == quit.clone().id() {
          info!("quitting");

          *control_flow = ControlFlow::Exit;
        }

        // Checking to see if a new device was selected
        //
        // If a new device was selected, update the icon and update the menu
        // accordingly.
        if devices.iter().any(|d| d.clone().id() == menu_id) {
          for device in &mut devices {
            if menu_id == device.clone().id() {
              debug!("selected device: {}", device.clone().title());
              device.set_selected(true);
              // Ellipsis icon to indicate background process
              system_tray
                .lock()
                .unwrap()
                .set_icon(Self::force_icon("80085"));
              trace!("updating system tray icon from intent");

              // If the selected device is the dummy device, set a dummy icon
              if device.0.title() == "Dummy (Debug)" {
                system_tray
                  .lock()
                  .unwrap()
                  .set_icon(Self::force_icon("43770"));
              } else {
                system_tray
                  .lock()
                  .unwrap()
                  .set_icon(Self::icon(&Some(device.0.title())));
              }

              trace!("updated system tray icon from intent");
              system_tray.lock().unwrap().set_tooltip(&format!(
                "elem (updating {} from intent)",
                device.0.title()
              ));
              local_self.lock().unwrap().selected_device_display_name =
                Some(device.0.title());
              system_tray
                .lock()
                .unwrap()
                .set_tooltip(&format!("elem ({})", device.0.title()));
              info!(
                "completed device selection ({}) and associated tasks",
                device.0.title()
              );
            } else {
              device.set_selected(false);
            }
          }
        }
      }
    });
  }
}
