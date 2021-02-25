// vinylla - (c) 2021 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

    fn normalize_sample_to_size(&self, sample: i16) -> usize {
        let sample = f32::from(sample) / -(i16::MIN as f32);
        ((self.half_size as i16) - (sample * ((self.half_size - 1) as f32)) as i16) as usize
    }

    const fn coordinate_to_index(&self, x: usize, y: usize) -> usize {
        x * self.size + y
    }

    pub fn draw_sample(&mut self, buffer: &mut [u8], size: usize, left: i16, right: i16) {
        assert_eq!(buffer.len(), size * size);

        if self.samples_drawn == self.decay_interval {
            self.decay(buffer, size);
            self.samples_drawn = 0;
        } else {
            self.samples_drawn += 1;
        }

        // Calculate coordinate in range [0, size - 1]
        let x = self.normalize_sample_to_size(left);
        let y = self.normalize_sample_to_size(right);

        // Draw pixel
        let index = self.coordinate_to_index(x, y);
        buffer[index] = u8::MAX;
    }
}

#[cfg(test)]
mod tests {
    use super::Visualizer;

    #[test]
    fn test_normalize() {
        const SIZE: usize = 400;
        let visualizer = Visualizer::new(SIZE);

        for sample in i16::MIN..=i16::MAX {
            let coord = visualizer.normalize_sample_to_size(sample);
            assert!(coord < SIZE);
        }
    }

    #[test]
    fn test_coordinate() {
        const SIZE: usize = 400;
        let visualizer = Visualizer::new(SIZE);

        for x in 0..SIZE - 1 {
            for y in 0..SIZE - 1 {
                let index = visualizer.coordinate_to_index(x, y);
                assert!(index < SIZE * SIZE);
            }
        }
    }
}
