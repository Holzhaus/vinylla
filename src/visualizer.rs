#[derive(Debug)]
pub struct Visualizer {
    size: usize,
    half_size: usize,
    samples_drawn: usize,
    decay_interval: usize,
    decay_factor: f32,
}

impl Visualizer {
    const DECAY_INTERVAL: usize = 50;
    const DECAY_FACTOR: f32 = 0.95;

    pub fn new(size: usize) -> Self {
        assert!(size > 10);

        let half_size = size / 2;
        assert_eq!(half_size * 2, size);

        Visualizer {
            size,
            half_size,
            samples_drawn: 0,
            decay_interval: Self::DECAY_INTERVAL,
            decay_factor: Self::DECAY_FACTOR,
        }
    }

    pub fn decay(&self, buffer: &mut [u8], size: usize) {
        let num_pixels = size * size;
        buffer
            .iter_mut()
            .take(num_pixels)
            .for_each(|x| *x = (f32::from(*x) * Self::DECAY_FACTOR) as u8);
    }

    pub fn draw_sample(&mut self, buffer: &mut [u8], size: usize, left: i16, right: i16) {
        assert_eq!(buffer.len(), size * size);

        if self.samples_drawn == self.decay_interval {
            self.decay(buffer, size);
            self.samples_drawn = 0;
        } else {
            self.samples_drawn += 1;
        }

        // Normalize to range [-1.0, 1.0]
        let x = f32::from(left) / (i16::MAX as f32);
        let y = f32::from(right) / (i16::MAX as f32);

        // Calculate coordinate in range [0, size]
        let x = ((self.half_size as i16) - (x * (self.half_size as f32)) as i16) as usize;
        let y = ((self.half_size as i16) - (y * (self.half_size as f32)) as i16) as usize;

        // Draw pixel
        if x != self.half_size && y != self.half_size {
            let index = x * size + y;
            buffer[index] = u8::MAX;
        }
    }
}
