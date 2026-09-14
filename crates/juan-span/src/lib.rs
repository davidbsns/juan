#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Span(pub u32);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        let start = (start as u32) & 0xF_FFFF;
        let len = ((end - start as usize) as u32) & 0xFFF;

        Self((start << 12) | len)
    }

    // Returns the Start and Length of the span.
    pub fn unpack(&self) -> (u32, u32) {
        let span = self.0;
        let start = span >> 12;
        let len = span & 0xFFF;

        (start, len)
    }

    // TODO: maybe i should make this a trait
    // Returns the Start and Length of the span.
    pub fn unpack_usize(&self) -> (usize, usize) {
        let span = self.0;
        let start = span >> 12;
        let len = span & 0xFFF;

        (start as usize, len as usize)
    }
}
