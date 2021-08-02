use crate::defer::DeferredIdent;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct Name {
    snake: Buffer,
    camel: Buffer,
}

impl Name {
    // snake: replace all hyphens with underscores
    // camel: split on hyphens and underscores, uppercase first byte of each word
    pub(crate) fn new(s: &str) -> Option<Name> {
        let bytes = validate(s)?;
        let mut snake = Buffer::new();
        snake.extend(bytes);
        for b in snake.iter_mut() {
            if *b == b'-' {
                *b = b'_';
            }
        }
        let mut camel = Buffer::new();
        for word in bytes.split(|&b| b == b'_' || b == b'-') {
            if let Some(&b) = word.first() {
                camel.push(b.to_ascii_uppercase());
                if let Some(bs) = word.get(1..) {
                    camel.extend(bs);
                }
            }
        }
        Some(Name { snake, camel })
    }
    pub(crate) fn snake(&self) -> DeferredIdent<'_> {
        // SAFETY: `Name::new` ensures that all source bytes match `a-z0-9_-`, and 
        // all subsequent buffer writes use bytes that also match said pattern
        unsafe { DeferredIdent(std::str::from_utf8_unchecked(&self.snake)) }
    }
    pub(crate) fn camel(&self) -> DeferredIdent<'_> {
        // SAFETY: `Name::new` ensures that all source bytes match `a-z0-9_-`, and 
        // all subsequent buffer writes use bytes that also match said pattern
        unsafe { DeferredIdent(std::str::from_utf8_unchecked(&self.camel)) }
    }
}

fn validate(s: &str) -> Option<&[u8]> {
    let bytes = s.as_bytes();
    (1 <= bytes.len()
        && bytes.len() <= MAX_LEN
        && bytes
            .iter()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-')))
    .then(|| bytes)
}

// NOTE: camel-case might be shorter by a few characters
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.camel.deref() == other.camel.deref()
    }
}

impl Eq for Name {}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.camel.deref().hash(state);
    }
}

// https://discord.com/developers/docs/interactions/slash-commands#registering-a-command
const MAX_LEN: usize = 32;

#[derive(Debug)]
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
