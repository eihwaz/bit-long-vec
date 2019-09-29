/// Vector with fixed bit sized values stored in long.
pub struct BitLongVec {
    /// Capacity of array.
    pub capacity: usize,
    /// Bits per value in internal storage.
    pub bits_per_value: u8,
    /// Maximum possible stored value.
    pub max_possible_value: u64,
    /// Internal storage for values.
    pub data: Vec<u64>,
}

impl BitLongVec {
    pub fn with_fixed_capacity(capacity: usize, bits_per_value: u8) -> Self {
        assert!(64 > bits_per_value, "Bit per value must be less than 64");

        let longs_required = ((capacity * bits_per_value as usize) as f64 / 64.0).ceil() as usize;
        let max_possible_value = (1 << bits_per_value as u64) - 1;
        let data = vec![0u64; longs_required]; // <- Fastest way to initialize a vector.

        BitLongVec {
            capacity,
            bits_per_value,
            max_possible_value,
            data,
        }
    }

    pub fn from_data(data: Vec<u64>, capacity: usize, bits_per_value: u8) -> Self {
        let max_possible_value = (1 << bits_per_value as u64) - 1;

        BitLongVec {
            capacity,
            bits_per_value,
            max_possible_value,
            data,
        }
    }

    pub fn set(&mut self, index: usize, value: u64) {
        let bit_index = index * self.bits_per_value as usize;
        let long_bit_index = bit_index % 64;
        let long_index = bit_index / 64;

        self.data[long_index] &= !(self.max_possible_value << long_bit_index as u64);
        self.data[long_index] |= value << long_bit_index as u64;

        // Value overlaps in the next long.
        if long_bit_index + self.bits_per_value as usize > 64 {
            let bits_written = 64 - long_bit_index;
            let bits_remaining = self.bits_per_value as usize - bits_written;

            let remainder_max_possible_value = (1 << bits_remaining as u64) - 1;

            self.data[long_index + 1] &= !(remainder_max_possible_value);
            self.data[long_index + 1] |= value >> bits_written as u64;
        }
    }

    pub fn get(&self, index: usize) -> u64 {
        let bit_index = index * self.bits_per_value as usize;
        let long_bit_index = bit_index % 64;
        let long_index = bit_index / 64;

        let long_value = self.data[long_index];
        let mut value = long_value >> long_bit_index as u64;

        // Value overlaps in the next long.
        if long_bit_index + self.bits_per_value as usize > 64 {
            let next_long_value = self.data[long_index + 1];
            value |= next_long_value << 64 - long_bit_index as u64;
        }

        value & self.max_possible_value
    }
}

#[test]
fn test_longs_required() {
    let data = vec![
        (2048, 4, 128),
        (4096, 4, 256),
        (2048, 8, 256),
        (4096, 8, 512),
        (4096, 14, 896),
    ];

    for (capacity, bits_per_value, expected_length) in data {
        let vec = BitLongVec::with_fixed_capacity(capacity, bits_per_value);

        assert_eq!(vec.data.len(), expected_length);
        assert_eq!(vec.data.capacity(), expected_length);
    }
}

#[test]
fn test_max_possible_value() {
    let data = vec![(4, 15), (5, 31), (6, 63), (7, 127), (8, 255), (14, 16_383)];

    for (bits_per_value, expected_max_possible_value) in data {
        let vec = BitLongVec::with_fixed_capacity(1, bits_per_value);
        assert_eq!(vec.max_possible_value, expected_max_possible_value);
    }
}

#[test]
fn test_set() {
    let mut vec = BitLongVec::with_fixed_capacity(48, 4);

    // long 1: [1, 2, 3, 4, 0, 0, 0, 0]
    // long 2: [5, 6, 7, 8, 0, 0, 0, 0]
    // long 3: [9, 10, 11, 12, 0, 0, 0, 0]
    for long_index in 0..3 {
        for long_byte_index in 0..4 {
            let index = long_index * 16 + long_byte_index;
            let value = (long_index * 4 + long_byte_index + 1) as u64;

            vec.set(index, value);
        }
    }

    assert_eq!(vec.data, vec![17185, 34661, 52137]);
}

#[test]
fn test_set_overlap() {
    let mut vec = BitLongVec::with_fixed_capacity(9, 14);

    for index in 0..9 {
        vec.set(index, (15_000 + index) as u64);
    }

    assert_eq!(vec.data, vec![11306972589037353624, 4224634284506261370]);
}

#[test]
fn test_set_clean_bits() {
    let mut vec = BitLongVec::from_data(vec![2762], 3, 4);
    vec.set(1, 0);

    assert_eq!(vec.data[0], 2570)
}

#[test]
fn test_set_overlap_clean_bits() {
    let data = vec![11306972589037353624, 4224634284506261370];
    let mut vec = BitLongVec::from_data(data, 9, 14);
    vec.set(4, 0);

    assert_eq!(vec.data[0], 65987919120595608);
    assert_eq!(vec.data[1], 4224634284506261312);
}

#[test]
fn test_set_change_bits() {
    let mut vec = BitLongVec::from_data(vec![2762], 3, 4);
    vec.set(1, 8);

    assert_eq!(vec.data[0], 2698);
}

#[test]
fn test_set_overlap_change_bits() {
    let data = vec![11306972589037353624, 4224634284506261370];
    let mut vec = BitLongVec::from_data(data, 9, 14);
    vec.set(4, 8);

    assert_eq!(vec.data[0], 642448671424019096);
    assert_eq!(vec.data[1], 4224634284506261312);
}

#[test]
fn test_get() {
    let data = vec![17185, 34661, 52137];
    let vec = BitLongVec::from_data(data, 48, 4);

    // long 1: [1, 2, 3, 4, 0, 0, 0, 0]
    // long 2: [5, 6, 7, 8, 0, 0, 0, 0]
    // long 3: [9, 10, 11, 12, 0, 0, 0, 0]
    for long_index in 0..3 {
        for long_byte_index in 0..4 {
            let index = long_index * 16 + long_byte_index;
            let value = (long_index * 4 + long_byte_index + 1) as u64;

            assert_eq!(vec.get(index), value)
        }
    }
}

#[test]
fn test_get_overlap() {
    let data = vec![11306972589037353624, 4224634284506261370];
    let vec = BitLongVec::from_data(data, 9, 14);

    for index in 0..9 {
        assert_eq!(vec.get(index), 15_000 + index as u64);
    }
}
