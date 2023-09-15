#[derive(Clone, Copy)]
pub struct Rgba {
    pub rgba: [u8; 4],
}

impl Rgba {
    pub fn black() -> Self {
        Self {
            rgba: [0, 0, 0, 255],
        }
    }

    pub fn dark() -> Self {
        Self {
            rgba: [84, 84, 84, 255],
        }
    }

    pub fn light() -> Self {
        Self {
            rgba: [168, 168, 168, 255],
        }
    }

    pub fn white() -> Self {
        Self {
            rgba: [255, 255, 255, 255],
        }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self { rgba: [0; 4] }
    }
}
