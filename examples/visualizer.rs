//! Reads a Serato Control CD 1.0.0 WAV file and show the "donut" visualizer.
//!
//! The WAV file can be downloaded from:
//! https://serato.com/controlcd/downloads/zip
//!
//! You can run this using:
//!
//! ```bash
//! $ cargo run --example visualizer -- /path/to/Serato\ Control\ CD.wav
//! ```
//!
//! Note that this will panic when the end of the file is reached.

use hound::WavReader;
use sdl2::{
    event::Event,
    pixels::{Color, PixelFormatEnum},
    render::Canvas,
    video::Window,
};
use std::env;
use vinylla::{Timecode, Visualizer, SERATO_CONTROL_CD_1_0_0};

const PIXEL_SIZE: usize = 400;

fn main() {
    let path = env::args().nth(1).expect("No file given");

    println!("File: {}", path);
    let mut reader = WavReader::open(&path).unwrap();
    let mut samples = reader.samples::<i16>().map(|x| x.unwrap());
    let mut timecode = Timecode::new(&SERATO_CONTROL_CD_1_0_0, 44100.0);

    // Set up SDL window and Texture that we can draw on
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    //video_subsystem.gl_set_swap_interval(SwapInterval::Immediate).unwrap();
    let window = video_subsystem
        .window("Example", PIXEL_SIZE as u32, PIXEL_SIZE as u32)
        .build()
        .unwrap();
    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_static(
            PixelFormatEnum::RGB332,
            PIXEL_SIZE as u32,
            PIXEL_SIZE as u32,
        )
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut pixels: [u8; PIXEL_SIZE * PIXEL_SIZE] = [0; PIXEL_SIZE * PIXEL_SIZE];
    let mut visualizer = Visualizer::new(PIXEL_SIZE);

    let mut i = 0;
    let mut samples_read = false;
    'running: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        let left = samples.next().expect("failed to read left sample");
        let right = samples.next().expect("failed to read right sample");
        if !samples_read && left == 0 && right == 0 {
            continue;
        }
        samples_read = true;
        if let Some((bit, position)) = timecode.process_channels(left, right) {
            println!("{:10}: Bit {} => Position {:?}", i, bit as u8, position);
        }

        visualizer.draw_sample(&mut pixels, PIXEL_SIZE, left, right);
        texture.update(None, &pixels, PIXEL_SIZE).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        i += 1;
    }
}
