use num_traits::AsPrimitive;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Span(u32);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        let len = end.saturating_sub(start);

        // TODO: figure out whether or not this'll become an issue
        // if so, ill have to expand to 2 u32s
        // i wont return a result for now as thats gonna be annoying
        assert!(len <= 0xFFF, "Length {} exceeds limit! (4095)", len);
        assert!(
            start <= 0xF_FFFF,
            "Length {} exceeds limit! (1,048,575)",
            start
        );

        let len = len as u32;
        let start = start as u32;

        Self((start << 12) | len)
    }

    // Returns the Start and **Length** of the span.
    pub fn unpack<T>(&self) -> (T, T)
    where
        T: Copy + 'static,
        u32: AsPrimitive<T>,
    {
        let span = self.0;
        let start = span >> 12;
        let len = span & 0xFFF;

        (start.as_(), len.as_())
    }
}
