pub struct Bitboard(pub u64);

impl Bitboard {
    // Boş bir bitboard oluşturur
    pub fn new() -> Self {
        Bitboard(0)
    }

    // Belirli bir karedeki biti 1 yapar
    pub fn set_square(&mut self, square: u8) {
        self.0 |= 1 << square;
    }

    // Belirli bir karedeki biti kontrol eder
    pub fn get_square(&self, square: u8) -> bool {
        (self.0 >> square) & 1 == 1
    }
}