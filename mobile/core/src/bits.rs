pub struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn read(&mut self, bit_count: u32) -> u32 {
        (0..bit_count).fold(0, |value, _| {
            let bit = (self.bytes[self.position / 8] >> (7 - self.position % 8)) & 1;
            self.position += 1;
            value << 1 | u32::from(bit)
        })
    }

    pub fn read_flag(&mut self) -> bool {
        self.read(1) == 1
    }

    pub fn skip(&mut self, bit_count: u32) {
        self.position += bit_count as usize;
    }

    pub fn read_exp_golomb(&mut self) -> u32 {
        let mut leading_zero_count = 0;
        while !self.read_flag() {
            leading_zero_count += 1;
        }
        (1 << leading_zero_count) - 1 + self.read(leading_zero_count)
    }

    pub fn read_signed_exp_golomb(&mut self) -> i32 {
        let value = self.read_exp_golomb() as i32;
        if value % 2 == 1 { (value + 1) / 2 } else { -value / 2 }
    }
}
