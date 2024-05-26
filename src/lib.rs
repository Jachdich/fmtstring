extern crate termion;
use std::ops::{Index, IndexMut, Range};

#[derive(Copy, Clone, Debug, Hash, std::cmp::Eq)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub default: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct FmtChar {
    pub ch: char,
    pub fg: Colour,
    pub bg: Colour,
}

#[derive(Clone, Debug)]
pub struct FmtString {
    container: Vec<FmtChar>,
}

pub enum Ground {
    Foreground,
    Background,
}

impl Index<Range<usize>> for FmtString {
    type Output = [FmtChar];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.container[range]
    }
}

impl Colour {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            default: false,
        }
    }
    pub fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            default: true,
        }
    }

    pub fn to_string(&self, ground: Ground) -> String {
        if self.default {
            match ground {
                Ground::Foreground => termion::color::Reset.fg_str().to_owned(),
                Ground::Background => termion::color::Reset.bg_str().to_owned(),
            }
        } else {
            match ground {
                Ground::Foreground => termion::color::Rgb(self.r, self.g, self.b).fg_string(),
                Ground::Background => termion::color::Rgb(self.r, self.g, self.b).bg_string(),
            }
        }
    }
}
impl std::cmp::PartialEq for Colour {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b && self.default == other.default
    }
}

impl Index<usize> for FmtString {
    type Output = FmtChar;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.container[idx]
    }
}

impl IndexMut<usize> for FmtString {
    fn index_mut(&mut self, idx: usize) -> &mut FmtChar {
        &mut self.container[idx]
    }
}

fn read_until(ch: char, data: &Vec<char>, mut pos: usize) -> (Vec<char>, usize) {
    if pos >= data.len() - 1 {
        return (vec![], pos);
    }
    let start = pos;
    while pos < data.len() && data[pos] != ch {
        pos += 1;
    }
    if pos >= data.len() - 1 {
        return (data[start..pos].to_vec(), pos);
    }
    (data[start..pos].to_vec(), pos)
}

impl FmtString {
    pub fn from_str(data: &str) -> Self {
        Self::from_str_colour(data, Colour::default(), Colour::default())
    }

    pub fn from_str_colour(data: &str, fg: Colour, bg: Colour) -> Self {
        let mut buf: Vec<FmtChar> = Vec::with_capacity(data.len());
        for ch in data.chars() {
            buf.push(FmtChar { ch, fg, bg });
        }

        Self { container: buf }
    }

    pub fn from_ansi_string(data: String) -> FmtString {
        let data: Vec<char> = data.chars().collect();
        let mut out: Vec<FmtChar> = Vec::new();
        let mut pos = 0;
        let mut bg = Colour::default();
        let mut fg = Colour::default();

        while pos < data.len() {
            if data[pos] == '\u{001b}' {
                pos += 2;
                if data[pos..pos + 2].to_vec().into_iter().collect::<String>() == "0m" {
                    bg = Colour::default();
                    fg = Colour::default();
                    pos += 2;
                    continue;
                }
                if data[pos..pos + 3].to_vec().into_iter().collect::<String>() == "39m" {
                    bg = Colour::default();
                    fg = Colour::default();
                    pos += 3;
                    continue;
                }
                if data[pos..pos + 3].to_vec().into_iter().collect::<String>() == "49m" {
                    bg = Colour::default();
                    fg = Colour::default();
                    pos += 3;
                    continue;
                }
                let (first_num, npos) = read_until(';', &data, pos);
                pos = npos;
                let (_second_num, npos) = read_until(';', &data, pos + 1);
                pos = npos;

                let (rv, npos) = read_until(';', &data, pos + 1);
                pos = npos;
                let (gv, npos) = read_until(';', &data, pos + 1);
                pos = npos;
                let (bv, npos) = read_until('m', &data, pos + 1);
                pos = npos;
                let r = rv.into_iter().collect::<String>().parse::<u8>().unwrap();
                let g = gv.into_iter().collect::<String>().parse::<u8>().unwrap();
                let b = bv.into_iter().collect::<String>().parse::<u8>().unwrap();
                pos += 1;
                let s: String = first_num.into_iter().collect();
                if s == "38" {
                    fg = Colour::from_rgb(r, g, b);
                } else if s == "48" {
                    bg = Colour::from_rgb(r, g, b);
                }
            } else {
                out.push(FmtChar {
                    ch: data[pos],
                    fg: fg,
                    bg: bg,
                });
                pos += 1;
            }
        }
        FmtString { container: out }
    }

    pub fn to_optimised_string(&self) -> String {
        // guess at the capacity, better than starting at 0
        let mut out = String::with_capacity(self.container.len() * 8);
        let mut last_fg = Colour::default();
        let mut last_bg = Colour::default();
        for ch in &self.container {
            if ch.fg != last_fg {
                out.push_str(&ch.fg.to_string(Ground::Foreground));
                last_fg = ch.fg;
            }
            if ch.bg != last_bg {
                out.push_str(&ch.bg.to_string(Ground::Background));
                last_bg = ch.bg;
            }
            out.push(ch.ch);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let original = include_str!("test2.txt").to_owned();
        // let ol = original.len();
        // let a = FmtString::from_ansi_string(original);
        let mut a = FmtString::from_str("▒I am a formatted string!");
        a[0].bg = Colour::from_rgb(255, 0, 0);
        // a[3].fg = Colour::from_rgb(255, 0, 0);
        let optimised = a.to_optimised_string();
        println!("{}", optimised);
    }
}
