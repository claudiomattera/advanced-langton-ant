// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::error;
use std::fmt;

use image::{ImageBuffer, Rgba, RgbaImage};

use crate::palette::Palette;

#[derive(Debug)]
pub struct Board {
    size: u32,
    data: Vec<u8>,
}

impl Board {
    pub fn new(size: u32) -> Self {
        Board {
            size,
            data: vec![0; (size * size) as usize],
        }
    }

    pub fn set(self: &mut Self, coordinates: (u32, u32), value: u8) {
        let i = self.xy_to_index(coordinates);
        self.data[i] = value;
    }

    pub fn get(self: &Self, coordinates: (u32, u32)) -> u8 {
        let i = self.xy_to_index(coordinates);
        self.data[i]
    }

    fn xy_to_index(self: &Self, (x, y): (u32, u32)) -> usize {
        (x * self.size + y) as usize
    }

    pub fn to_image(self: &Self, palette: &Palette) -> RgbaImage {
        ImageBuffer::from_fn(
            self.size as u32,
            self.size as u32,
            |x, y| {
                let index = self.get((x, y));
                Rgba::<u8>(palette.get_color(index as usize))
            }
        )
    }

    pub fn walk_langton_ant(self: &mut Self, rule: &Rule, iterations: u64) {
        self.walk_and_save_langton_ant(rule, iterations, Box::new(|_, _| {}))
    }

    pub fn walk_and_save_langton_ant(
                self: &mut Self,
                rule: &Rule,
                iterations: u64,
                mut frame_saver: Box<dyn FnMut(&Self, u64) -> ()>,
            ) {
        let mut position = (self.size / 2, self.size / 2);
        let mut direction = Direction::North;
        let mut last_iteration = iterations;
        for i in 0..iterations {
            // Flip current cell
            let new_value = rule.next_value(self.get(position));
            self.set(position, new_value);

            // Compute new position
            match self.next_position(&direction, position) {
                None => {
                    last_iteration = i;
                    break;
                }
                Some(new_position) => position = new_position,
            }

            // Compute new direction
            let new_direction = rule.next_direction(self.get(position), direction);
            direction = new_direction;

            // Leave the decision to save the current frame to the caller
            frame_saver(self, i);
        }

        info!("Last iteration: {}", last_iteration);
    }

    fn next_position(self: &Self, direction: &Direction, (x, y): (u32, u32)) -> Option<(u32, u32)> {
        match direction {
            Direction::North => if y == self.size - 1 {None} else {Some((x, y + 1))},
            Direction::South => if y == 0 {None} else {Some((x, y - 1))},
            Direction::East => if x == self.size - 1 {None} else {Some((x + 1, y))},
            Direction::West => if x == 0 {None} else {Some((x - 1, y))},
        }
    }
}

#[derive(Debug)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn turn_left(self: Self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    pub fn turn_right(self: Self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Element {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Rule {
    elements: Vec<Element>,
}

impl Rule {
    pub fn new(desc: &str) -> Result<Self, RuleParseError> {
        let mut elements = Vec::with_capacity(desc.len());
        for character in desc.chars() {
            match character {
                'L' => elements.push(Element::Left),
                'R' => elements.push(Element::Right),
                _ => return Err(RuleParseError),
            }
        }
        Ok(Rule { elements })
    }

    pub fn len(self: &Self) -> usize {
        self.elements.len()
    }

    pub fn next_value(self: &Self, value: u8) -> u8 {
        (value + 1) % self.elements.len() as u8
    }

    pub fn next_direction(self: &Self, value: u8, direction: Direction) -> Direction {
        match self.elements[value as usize] {
            Element::Left => direction.turn_left(),
            Element::Right => direction.turn_right(),
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &element in &self.elements {
            match element {
                Element::Left => write!(f, "L")?,
                Element::Right => write!(f, "R")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RuleParseError;

impl fmt::Display for RuleParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid rule string")
    }
}

impl error::Error for RuleParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
