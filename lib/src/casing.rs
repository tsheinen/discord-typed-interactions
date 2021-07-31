use std::ops::{Deref, DerefMut};
use crate::{mk_ident, Name};

// https://discord.com/developers/docs/interactions/slash-commands#registering-a-command
const MAX_LEN: usize = 32;

struct Buffer {
    buf: [u8; MAX_LEN],
    len: usize,
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            buf: [0_u8; MAX_LEN],
            len: 0,
        }
    }
    pub fn push(&mut self, b: u8) {
        self.buf[self.len] = b;
        self.len += 1;
    }
    pub fn extend(&mut self, bytes: &[u8]) {
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
    }
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Deref for Buffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf[..self.len]
    }
}

pub fn validate(s: &str) -> Option<&[u8]> {
    let bytes = s.as_bytes();
    (1 <= bytes.len()
        && bytes.len() <= 32
        && bytes
            .iter()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-')))
    .then(|| bytes)
}


fn to_upper(b: u8) -> u8 {
    if b'a' <= b && b <= b'z' {
        b - b'a' + b'A'
    } else {
        b
    }
}

// snake: replace all hyphens with underscores
// camel: split on hyphens and underscores, uppercase first byte of each word
pub(crate) fn mk_name(s: &str) -> Option<Name> {
    let bytes = validate(s)?;
    let mut buf = Buffer::new();
    buf.extend(bytes);
    for b in buf.iter_mut() {
        if *b == b'-' {
            *b = b'_';
        }
    }
    let snake = mk_ident(std::str::from_utf8(&buf).ok()?);
    buf.clear();
    for word in bytes.split(|&b| b == b'_' || b == b'-') {
        if let Some(&b) = word.first() {
            buf.push(to_upper(b));
            if let Some(bs) = word.get(1..) {
                buf.extend(bs);
            }
        }
    }
    let camel = mk_ident(std::str::from_utf8(&buf).ok()?);
    Some(Name { snake, camel })
}
