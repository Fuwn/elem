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

#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code,
  clippy::all,
  clippy::nursery,
  clippy::pedantic
)]
#![recursion_limit = "128"]

mod ascii_art;
mod logitech;
mod tray;

#[macro_use]
extern crate log;

fn main() {
  std::env::set_var("RUST_LOG", "elem=trace");
  pretty_env_logger::init();
  info!("starting elem");
  tray::Tray::new(std::env::args().nth(1)).run();
}
