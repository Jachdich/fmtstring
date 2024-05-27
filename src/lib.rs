use std::ops::{Index, IndexMut, Range};

#[derive(Copy, Clone, Debug, Hash, std::cmp::PartialEq)]
pub enum Colour {
    Rgb { r: u8, g: u8, b: u8 },
    Black,
    Blue,
    Cyan,
    Green,
    LightBlack,
    LightBlue,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
    Magenta,
    Red,
    White,
    Yellow,
    Default,
    None,
}

#[derive(Copy, Clone, Debug)]
pub struct FmtChar {
    pub ch: char,
    pub fg: Colour,
    pub bg: Colour,
}

impl FmtChar {
    pub fn width(&self) -> u16 {
        1
    }
}

#[derive(Clone, Debug)]
pub struct FmtString {
    container: Vec<FmtChar>,
    dirty: bool,
    cache: String,
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
        Self::Rgb { r, g, b }
    }
    pub fn default() -> Self {
        Self::Default
    }

    #[rustfmt::skip]
    pub fn to_string(&self, ground: Ground) -> String {
        // if self.default {
        //     match ground {
        //         Ground::Foreground => termion::color::Reset.fg_str().to_owned(),
        //         Ground::Background => termion::color::Reset.bg_str().to_owned(),
        //     }
        // } else {
        //     match ground {
        //         Ground::Foreground => termion::color::Rgb(self.r, self.g, self.b).fg_string(),
        //         Ground::Background => termion::color::Rgb(self.r, self.g, self.b).bg_string(),
        //     }
        // }
        let bg = match ground { Ground::Foreground => false, Ground::Background => true };
        match self {
            Self::Rgb { r, g, b } => if bg { termion::color::Rgb(*r, *g, *b).bg_string() } else { termion::color::Rgb(*r, *g, *b).fg_string() },
            Self::Black         => if bg { termion::color::Black.bg_str().into()        } else { termion::color::Black.fg_str().into() },
            Self::Blue          => if bg { termion::color::Blue.bg_str().into()         } else { termion::color::Blue.fg_str().into() },
            Self::Cyan          => if bg { termion::color::Cyan.bg_str().into()         } else { termion::color::Cyan.fg_str().into() },
            Self::Green         => if bg { termion::color::Green.bg_str().into()        } else { termion::color::Green.fg_str().into() },
            Self::LightBlack    => if bg { termion::color::LightBlack.bg_str().into()   } else { termion::color::LightBlack.fg_str().into() },
            Self::LightBlue     => if bg { termion::color::LightBlue.bg_str().into()    } else { termion::color::LightBlue.fg_str().into() },
            Self::LightGreen    => if bg { termion::color::LightGreen.bg_str().into()   } else { termion::color::LightGreen.fg_str().into() },
            Self::LightMagenta  => if bg { termion::color::LightMagenta.bg_str().into() } else { termion::color::LightMagenta.fg_str().into() },
            Self::LightRed      => if bg { termion::color::LightRed.bg_str().into()     } else { termion::color::LightRed.fg_str().into() },
            Self::LightWhite    => if bg { termion::color::LightWhite.bg_str().into()   } else { termion::color::LightWhite.fg_str().into() },
            Self::LightYellow   => if bg { termion::color::LightYellow.bg_str().into()  } else { termion::color::LightYellow.fg_str().into() },
            Self::Magenta       => if bg { termion::color::Magenta.bg_str().into()      } else { termion::color::Magenta.fg_str().into() },
            Self::Red           => if bg { termion::color::Red.bg_str().into()          } else { termion::color::Red.fg_str().into() },
            Self::Default       => if bg { termion::color::Reset.bg_str().into()        } else { termion::color::Reset.fg_str().into() },
            Self::White         => if bg { termion::color::White.bg_str().into()        } else { termion::color::White.fg_str().into() },
            Self::Yellow        => if bg { termion::color::Yellow.bg_str().into()       } else { termion::color::Yellow.fg_str().into() },
            Self::None => "".into(),
        }
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
        self.dirty = true; // could be modifying it
        &mut self.container[idx]
    }
}

impl From<String> for FmtString {
    fn from(item: String) -> Self {
        FmtString::from_str(&item)
    }
}

impl From<Vec<FmtChar>> for FmtString {
    fn from(item: Vec<FmtChar>) -> Self {
        Self {
            container: item,
            dirty: true,
            cache: "".into(),
        }
    }
}

impl From<&[FmtChar]> for FmtString {
    fn from(item: &[FmtChar]) -> Self {
        Self {
            container: item.to_vec(),
            dirty: true,
            cache: "".into(),
        }
    }
}

impl From<FmtString> for String {
    fn from(mut item: FmtString) -> String {
        if item.dirty {
            item.rebuild_cache();
        }
        item.cache
    }
}

impl std::fmt::Display for FmtChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.fg.to_string(Ground::Foreground),
            self.bg.to_string(Ground::Background),
            self.ch
        )
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

    pub fn new() -> Self {
        Self {
            container: Vec::new(),
            dirty: true,
            cache: "".into(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            container: Vec::with_capacity(capacity),
            dirty: true,
            cache: "".into(),
        }
    }

    pub fn push(&mut self, c: FmtChar) {
        self.container.push(c);
    }

    pub fn from_str_colour(data: &str, fg: Colour, bg: Colour) -> Self {
        let mut buf: Vec<FmtChar> = Vec::with_capacity(data.len());
        for ch in data.chars() {
            buf.push(FmtChar { ch, fg, bg });
        }

        Self {
            container: buf,
            dirty: true,
            cache: "".into(),
        }
    }

    pub fn to_str<'a>(&'a mut self) -> &'a str {
        if self.dirty {
            self.rebuild_cache();
        }
        &self.cache
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
                    fg,
                    bg,
                });
                pos += 1;
            }
        }
        FmtString {
            container: out,
            dirty: true,
            cache: "".into(),
        }
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

    fn rebuild_cache(&mut self) {
        if self.dirty {
            self.cache = self.to_optimised_string();
            self.dirty = false;
        }
    }
    pub fn concat(mut a: FmtString, b: FmtString) -> FmtString {
        a.container.extend(b.container.iter());
        a.dirty = true;
        a
    }
    pub fn len(&self) -> usize {
        self.container.len()
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
