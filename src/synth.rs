use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use dasp_signal::{self as signal, Signal};

const SAMPLE_RATE: u32 = 48_000;

#[derive(Debug)]
pub enum Control {
    Play,
    Pause,
    Exit,
}

pub fn init(rx: std::sync::mpsc::Receiver<Control>) -> Result<(), Box<dyn std::error::Error>> {
    // audio synthesis
    let host = cpal::default_host();

    // record desktop audio
    // let input_device = host
    //     .input_devices()?
    //     .find(|d| d.name().unwrap().contains("Stereo Mix"))
    //     .expect("'Stereo Mix' input device must be available to record desktop audio");

    // play back to default output device
    let output_device = host
        .default_output_device()
        .expect("default output device should be available");

    eprintln!("🎵 playing audio via '{}'", output_device.name().unwrap());

    let supported_config = output_device
        .supported_output_configs()?
        .next()
        .unwrap()
        .with_sample_rate(cpal::SampleRate(SAMPLE_RATE));
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    // SampleFormat::{I16, U16} are other common types
    if !matches!(sample_format, SampleFormat::F32) {
        panic!("unsupported sample format '{sample_format}'");
    }

    let osc = signal::rate(SAMPLE_RATE as f64)
        .const_hz(1.3) // base value is 1.0, inc/dec to change frequency
        .sine()
        // Waves are generated with an amplitude range of [-1.0, 1.0].
        // We want to instead generate them in the range of audible tones,
        // which is something like (0.0, 4000.0). In order to do so,
        // we have to crush the wave in half and shift it up a half step,
        // then multiply it by the maximum possible amplitude.
        //
        //   +1.0         _                          _
        //              /  \                       /  \     /
        //            /     \                    /     \  /
        //         ----------+---------       ----------_------
        //                    \    /
        //                     \  /
        //   -1.0               -
        //
        // This oscillator makes the classic Pac-Man sound!
        .map(|f| (f / 2.0 + 0.5) * 400.0);
    let mut wave = signal::rate(SAMPLE_RATE as f64).hz(osc).sine();

    let stream = output_device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.iter_mut() {
                    *frame = wave.next() as f32;
                }
            },
            |err| eprintln!("an error occurred on the output audio stream: {}", err),
            None,
        )
        .unwrap();

    loop {
        match rx.recv().unwrap() {
            Control::Play => stream.play().unwrap(),
            Control::Pause => stream.pause().unwrap(),
            Control::Exit => break,
        }
    }

    Ok(())
}
