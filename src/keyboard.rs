use std::collections::HashMap;

lazy_static! {
    pub static ref PC_KEY_MAP: HashMap<usize, u8> = {
        let mut map = HashMap::new();
        [
            (88, 0x0),
            (49, 0x1),
            (50, 0x2),
            (51, 0x3),
            (81, 0x4),
            (87, 0x5),
            (69, 0x6),
            (65, 0x7),
            (83, 0x8),
            (68, 0x9),
            (90, 0xA),
            (67, 0xB),
            (52, 0xC),
            (82, 0xD),
            (70, 0xE),
            (86, 0xF),
        ].iter().for_each(|t| {
            map.insert(t.0, t.1);
        });
        map
    };
}

// const PC_KEY_MAP: HashMap<u32, u8> = KEY_TUPLES.into_iter().cloned().collect();

// const CHIP8_KEYS: HashMap<u8, u32> = KEY_TUPLES.iter().cloned().map(|(a, b)| (b, a)).collect();

#[derive(Clone, Debug)]
pub struct KeyboardState {
    key_state: [bool; 16]
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {key_state: [false; 16]}
    }
    pub fn is_key_pressed(&self, k: &u8) -> bool {
        self.key_state[*k as usize]
    }
    pub fn key_down(&mut self, k: &u8) {
        self.key_state[*k as usize] = true;
    }
    pub fn key_up(&mut self, k: &u8) {
        self.key_state[*k as usize] = false;
    }
}