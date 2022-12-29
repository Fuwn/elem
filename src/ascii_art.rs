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

/// ASCII lettering from <http://www.patorjk.com/software/taag/#p=display&f=ANSI%20Regular&t=Type%20Something%20>

pub const HEIGHT: usize = 5;
const ONE: &str = r#" ██ 
███ 
 ██ 
 ██ 
 ██ "#;
const TWO: &str = r#"██████  
     ██ 
 █████  
██      
███████ "#;
const THREE: &str = r#"██████  
     ██ 
 █████  
     ██ 
██████  "#;
const FOUR: &str = r#"██   ██ 
██   ██ 
███████ 
     ██ 
     ██ "#;
const FIVE: &str = r#"███████ 
██      
███████ 
     ██ 
███████ "#;
const SIX: &str = r#" ██████  
██       
███████  
██    ██ 
 ██████  "#;
const SEVEN: &str = r#"███████ 
     ██ 
    ██  
   ██   
   ██   "#;
const EIGHT: &str = r#" █████  
██   ██ 
 █████  
██   ██ 
 █████  "#;
const NINE: &str = r#" █████  
██   ██ 
 ██████ 
     ██ 
 █████  "#;
const ZERO: &str = r#" ██████  
██  ████ 
██ ██ ██ 
████  ██ 
 ██████  "#;
const QUESTION_MARK: &str = r#"██████  
     ██ 
  ▄███  
  ▀▀    
  ██    "#;
const ELLIPSIS: &str = r#"         
         
         
         
██ ██ ██ "#;
const SMILEY_FACE: &str = r#"   ██  
██  ██ 
    ██ 
██  ██ 
   ██  "#;

/// Convert a number to ASCII art
fn number_to_art(number: u64) -> String {
  // Used for when an error occurs
  if number == 1337 {
    return QUESTION_MARK.to_string();
  } else if number == 80085 {
    // Used for when a background process is running
    return ELLIPSIS.to_string();
  } else if number == 43770 {
    // The battery level display for the dummy process
    return SMILEY_FACE.to_string();
  }

  let to_art_digit = |number: u64| match number {
    0 => ZERO,
    1 => ONE,
    2 => TWO,
    3 => THREE,
    4 => FOUR,
    5 => FIVE,
    6 => SIX,
    7 => SEVEN,
    8 => EIGHT,
    9 => NINE,
    _ => unreachable!(),
  };
  let mut art = String::new();

  // Splitting the number into its individual digits, then convert each digit to
  // it's ASCII art representation
  for i in 0..HEIGHT {
    for digit in number
      .to_string()
      .chars()
      .map(|c| {
        u64::from(c.to_digit(10).expect(
          "couldn't convert character to digit, this should never happen",
        ))
      })
      .collect::<Vec<_>>()
    {
      art.push_str(
        to_art_digit(digit)
          .lines()
          .nth(i)
          .expect("invalid line from digit art, this should never happen"),
      );
    }

    art.push('\n');
  }

  // Removing the last newline to get rid of the last empty line
  art.pop();

  art
}

pub fn number_to_image(number: u64) -> Vec<u8> {
  let art = number_to_art(number);
  let mut image = vec![];

  // Iterating over each character in the ASCII art, and converting it into a
  // transparent or filled pixel
  for pixel in art
    // Replacing these characters with different characters isn't at all needed
    // (except for the newline character), but it makes the following
    // process a little more readable.
    .replace(' ', "a")
    .replace('█', "b")
    .replace('\n', "")
    .as_bytes()
  {
    if pixel == &97 {
      // A transparent pixel
      image.push(0);
      image.push(0);
      image.push(0);
      image.push(0);
    } else if pixel == &98 {
      // A solid white pixel
      image.push(255u8);
      image.push(255u8);
      image.push(255u8);
      image.push(255u8);
    } else {
      unreachable!();
    }
  }

  // Create an image from the pixel data
  lodepng::encode_memory(
    &image,
    art.lines().next().unwrap().chars().count(),
    HEIGHT,
    lodepng::ColorType::RGBA,
    8,
  )
  .unwrap_or_else(|_| panic!("unable to encode digit {number}"))
}
