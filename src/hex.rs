pub fn encode(slice: &[u8]) -> String {
    BytesToHexChars::new(slice).collect()
}

const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";

struct BytesToHexChars<'a> {
    inner: ::core::slice::Iter<'a, u8>,
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8]) -> BytesToHexChars<'a> {
        BytesToHexChars {
            inner: inner.iter(),
            next: None,
        }
    }
}

impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = HEX_CHARS_LOWER[(byte >> 4) as usize] as char;
                self.next = Some(HEX_CHARS_LOWER[(byte & 0x0F) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> std::iter::ExactSizeIterator for BytesToHexChars<'a> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}
