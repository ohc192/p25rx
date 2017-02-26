use std::io::Write;
use std::sync::mpsc::Receiver;
use std;

use imbe::consts::SAMPLES_PER_FRAME;
use imbe::decode::ImbeDecoder;
use imbe::frame::ReceivedFrame;
use map_in_place::MapInPlace;
use p25::voice::frame::VoiceFrame;

pub enum AudioEvent {
    VoiceFrame(VoiceFrame),
    EndTransmission,
}

pub struct AudioTask<W: Write> {
    audio: AudioOutput<W>,
    events: Receiver<AudioEvent>,
}

impl<W: Write> AudioTask<W> {
    pub fn new(audio: AudioOutput<W>, events: Receiver<AudioEvent>) -> Self {
        AudioTask {
            audio: audio,
            events: events,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.events.recv().expect("unable to receive audio event") {
                AudioEvent::VoiceFrame(vf) => self.audio.play(&vf),
                AudioEvent::EndTransmission => {
                   self.audio.flush();
                   self.audio.reset();
                },
            }
        }
    }
}

pub struct AudioOutput<W: Write> {
    stream: W,
    imbe: ImbeDecoder,
}

impl<W: Write> AudioOutput<W> {
    pub fn new(stream: W) -> Self {
        AudioOutput {
            stream: stream,
            imbe: ImbeDecoder::new(),
        }
    }

    pub fn reset(&mut self) {
        self.imbe = ImbeDecoder::new();
    }

    pub fn play(&mut self, frame: &VoiceFrame) {
        let frame = ReceivedFrame::new(frame.chunks, frame.errors);

        let mut samples = [0.0; SAMPLES_PER_FRAME];
        self.imbe.decode(frame, &mut samples);

        // TODO: AGC or proper volume normalization.
        samples.map_in_place(|&s| s / 8192.0);

        self.stream.write_all(unsafe {
            std::slice::from_raw_parts(
                samples.as_ptr() as *const u8,
                samples.len() * std::mem::size_of::<f32>()
            )
        }).expect("unable to write audio samples");
    }

    pub fn flush(&mut self) {
        self.stream.flush().expect("unable to flush audio samples")
    }
}
