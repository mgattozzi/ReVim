extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate termion;

use clap::{ App, Arg };
use termion::cursor;
use termion::screen::*;
use termion::raw::{ RawTerminal, IntoRawMode };
use termion::event::Key;
use termion::input::TermRead;
use termion::clear;

use std::process::exit;
use std::io::{ BufReader, BufRead, Stdout, Write, stdout, stdin, stderr };
use std::fs::File;

mod error;
use error::*;

fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "Error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "Caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "Backtrace: {:?}", backtrace).expect(errmsg);
        }

        // Return to the terminal as it was before opening up rvim
        println!("{}", ToMainScreen);

        exit(1);
    } else {

        // Return to the terminal as it was before opening up rvim
        println!("{}", ToMainScreen);
    }
}

fn run() -> Result<()> {
    let matches = App::new("Re:Vim")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Michael Gattozzi <mgattozzi@gmail.com>")
        .about("Vim for the modern age. Rebuilt from the ground up.")
        .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(false)
                .index(1))
        .get_matches();

    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    write!(screen, "{}{}", clear::All, cursor::Goto(1,1))?;

    let current_buffer = Buffer::new(matches.value_of("INPUT"))?;
    current_buffer.draw_buffer(&mut screen)?;

    let stdin = stdin();

    let mut mode = Mode::Normal;
    for key in stdin.keys() {
        match mode {
            Mode::Normal => {
                match key? {
                    Key::Char('i') => mode = Mode::Insert,
                    Key::Char('v') => mode = Mode::Visual,
                    Key::Char('V') => mode = Mode::VisualLine,
                    Key::Ctrl('v') => mode = Mode::VisualBlock,
                    Key::Char(':') => mode = Mode::CommandLine,
                    Key::Char('h') => print!("{}", cursor::Left(1)),
                    Key::Char('j') => print!("{}", cursor::Down(1)),
                    Key::Char('k') => print!("{}", cursor::Up(1)),
                    Key::Char('l') => print!("{}", cursor::Right(1)),
                    Key::Char('q') => break,
                    _ => {},
                }
            },
            Mode::Insert => {
                match key? {
                    Key::Esc => mode = Mode::Normal,
                    Key::Char(c) => print!("{}", c),
                    _ => {},
                }
            },
            Mode::Visual => {
                match key? {
                    Key::Esc => mode = Mode::Normal,
                    _ => {},
                }
            },
            Mode::VisualBlock => {
                match key? {
                    Key::Esc => mode = Mode::Normal,
                    _ => {},
                }
            },
            Mode::VisualLine => {
                match key? {
                    Key::Esc => mode = Mode::Normal,
                    _ => {},
                }
            },
            Mode::CommandLine => {
                match key? {
                    Key::Esc => mode = Mode::Normal,
                    _ => {},
                }
            },
        }
        screen.flush()?;
    }

    Ok(())
}

enum Mode {
    Insert,
    Normal,
    Visual,
    VisualBlock,
    VisualLine,
    CommandLine,
}

pub struct Buffer {
    pub lines: Vec<String>,
    pub cursor: (u16, u16),
}

impl Buffer {
    pub fn new(input_file: Option<&str>) -> Result<Self> {
        match input_file {
            Some(file) => {
                let reader = BufReader::new(File::open(file)?);
                Ok(Buffer {
                    lines: reader
                            .lines()
                            .map(|x| {
                                // If the line was misread just make it blank
                                // Should probably handle this better in the
                                // future.
                                x.unwrap_or(String::new()).to_owned()
                            })
                            .collect::<Vec<String>>(),
                    cursor: (1,1)
                })
            }
            None => {
                Ok(Buffer {
                    lines: Vec::new(),
                    cursor: (1,1)
                })
            }
        }
    }

    pub fn draw_buffer(&self, screen: &mut AlternateScreen<RawTerminal<Stdout>>) -> Result<()> {
        for line in self.lines.iter() {
            writeln!(screen, "{}\r", line)?;
        }
        let (x,y) = self.cursor;
        write!(screen, "{}", cursor::Goto(x,y))?;

        screen.flush()?;

        Ok(())
    }

}
