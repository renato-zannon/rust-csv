#![crate_id = "csv"]
#![crate_type = "lib"]
#![desc = "CSV parser"]
#![license = "MIT"]

#![feature(phase)]
#[phase(syntax, link)] extern crate log;

use std::io;
use std::str;
use std::iter::Iterator;

pub type Row = Vec<~str>;

enum State {
  Continue,
  Wait,
  EOL
}

pub struct Parser<R> {
  count: uint,
  readlen: uint,
  delim: char,
  buffer: Vec<char>,
  acc: Vec<char>,
  row: Row,
  reader: R,
  state: State
}


pub fn init<R: io::Reader>(reader: R) -> Parser<R> {
  Parser {
    count: 0,
    readlen: 1024,
    delim: ',',
    buffer: vec!(),
    acc: vec!(),
    row: vec!(),
    reader: reader,
    state: Continue
  }
}

impl<R: Reader> Parser<R> {
  fn parse_next_char(&mut self) -> State {
    if self.buffer.len() == 0 {

      let mut bytes = [0, .. 1024];
      let optnbread = self.reader.read(bytes);
      if bytes.len() == 0 {
        debug!("0 bytes read");
        return Wait
      }

      match optnbread {
        Err(e)     => { debug!("opntbread error: {}", e); return Wait},
        Ok(nb)     => {
          debug!("optnbread {} bytes", nb);
          let s  = str::from_utf8(bytes);
          if s.is_some() {
            for el in s.unwrap().slice(0, nb).chars() {
              self.buffer.push(el);
            }
          }
        }
      }
    }

    let optc = self.buffer.shift();
    match optc {
      None    => { debug!("optc is none");return Wait},
      Some(c) => return self.parse_char(c)
    }
  }

  fn parse_char(&mut self, c: char) -> State {
    if c == self.delim {
        self.row.push(str::from_chars(self.acc.as_slice()));
        self.acc.clear();
        return Continue
    }

    match c {
      '\r' => Continue,
      '\n' => {
        self.row.push(str::from_chars(self.acc.as_slice()));
        self.acc.clear();
        EOL
      },
      _    => {
        self.acc.push(c);
        if self.buffer.len() == 0 {
          self.row.push(str::from_chars(self.acc.as_slice()));
          self.acc.clear();
        }
        Continue
      }
    }
  }

  fn extract_row(&mut self) -> Row {
    use std::mem::replace;

    replace(&mut self.row, Vec::new())
  }

  pub fn delim(&mut self, delim:char) {
    self.delim = delim;
  }
}

impl<R: Reader> Iterator<Row> for Parser<R> {
  fn next(&mut self) -> Option<Row> {
    loop {
      match self.parse_next_char() {
        EOL => {
          let row = self.extract_row();
          if row.len() > 0 {
            self.state = Continue;
            return Some(row);
          } else {
            self.state = EOL;
            return None;
          }
        }
        Continue => (),
        Wait => {
          self.state = Wait;
          let row = self.extract_row();
          if row.len() > 0 {
            return Some(row);
          }  else {
            return None
          }
        }
      }
    }
  }
}

