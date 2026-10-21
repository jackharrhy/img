use anyhow::{Context, Result, ensure};
use image::{AnimationDecoder, RgbaImage};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

pub const MAX_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 4096;
pub const MAX_FRAMES: usize = 1000;

#[derive(Clone, Debug)]
pub struct Frame {
    pub pixels: RgbaImage,
    pub delay_ms: u32,
}
#[derive(Clone, Debug)]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub repeat: gif::Repeat,
}
impl Animation {
    pub fn load(path: &Path) -> Result<Self> {
        ensure!(
            path.metadata()?.len() <= 64 * 1024 * 1024,
            "GIF file exceeds 64 MiB"
        );
        let metadata = gif::DecodeOptions::new().read_info(BufReader::new(File::open(path)?))?;
        let (w, h) = (metadata.width() as u32, metadata.height() as u32);
        dimensions(w, h)?;
        let repeat = metadata.repeat();
        drop(metadata);
        let decoder = image::codecs::gif::GifDecoder::new(BufReader::new(File::open(path)?))?;
        let mut frames = Vec::new();
        for frame in decoder.into_frames() {
            ensure!(frames.len() < MAX_FRAMES, "GIF exceeds {MAX_FRAMES} frames");
            ensure!(
                (frames.len() + 1) * w as usize * h as usize * 4 <= MAX_BYTES,
                "Decoded GIF exceeds 256 MiB"
            );
            let frame = frame.context("Decoding GIF frame")?;
            let (n, d) = frame.delay().numer_denom_ms();
            frames.push(Frame {
                delay_ms: n / d,
                pixels: frame.into_buffer(),
            });
        }
        ensure!(!frames.is_empty(), "GIF contains no frames");
        Ok(Self { frames, repeat })
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        ensure!(!self.frames.is_empty(), "No frames to export");
        let (w, h) = self.frames[0].pixels.dimensions();
        dimensions(w, h)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        {
            let mut writer = BufWriter::new(temp.as_file_mut());
            {
                let mut encoder = gif::Encoder::new(&mut writer, w as u16, h as u16, &[])?;
                encoder.set_repeat(self.repeat)?;
                for frame in &self.frames {
                    ensure!(
                        frame.pixels.dimensions() == (w, h),
                        "Inconsistent frame dimensions"
                    );
                    let mut rgba = frame.pixels.as_raw().clone();
                    let mut encoded =
                        gif::Frame::from_rgba_speed(w as u16, h as u16, &mut rgba, 10);
                    encoded.delay = ((frame.delay_ms + 5) / 10).min(u16::MAX as u32) as u16;
                    encoded.dispose = gif::DisposalMethod::Background;
                    encoder.write_frame(&encoded)?;
                }
            }
            writer.flush()?;
        }
        temp.persist(path).context("Saving output GIF")?;
        Ok(())
    }
    pub fn bytes(&self) -> usize {
        self.frames.iter().map(|f| f.pixels.as_raw().len()).sum()
    }
}
#[derive(Clone, Debug)]
pub enum Operation {
    Resize { width: u32, height: u32 },
    Reverse,
    Negate,
}
#[derive(Clone, Debug)]
pub struct Card {
    pub id: u64,
    pub enabled: bool,
    pub operation: Operation,
}
pub fn process(
    source: std::sync::Arc<Animation>,
    cards: &[Card],
) -> Result<Vec<std::sync::Arc<Animation>>> {
    let mut current = source;
    let mut total = current.bytes();
    let mut outputs = Vec::new();
    for card in cards {
        if card.enabled {
            let required = match card.operation {
                Operation::Resize { width, height } => {
                    dimensions(width, height)?;
                    width as usize * height as usize * 4 * current.frames.len()
                }
                Operation::Reverse | Operation::Negate => current.bytes(),
            };
            total += required;
            ensure!(
                total <= MAX_BYTES,
                "Operation stack exceeds 256 MiB; reduce dimensions or remove cards"
            );
            let next = match card.operation {
                Operation::Negate => Animation {
                    repeat: current.repeat,
                    frames: current
                        .frames
                        .iter()
                        .map(|frame| {
                            let mut frame = frame.clone();
                            for pixel in frame.pixels.pixels_mut() {
                                for channel in &mut pixel.0[..3] {
                                    *channel = 255 - *channel;
                                }
                            }
                            frame
                        })
                        .collect(),
                },
                Operation::Resize { width, height } => Animation {
                    repeat: current.repeat,
                    frames: current
                        .frames
                        .iter()
                        .map(|f| Frame {
                            delay_ms: f.delay_ms,
                            pixels: image::imageops::resize(
                                &f.pixels,
                                width,
                                height,
                                image::imageops::FilterType::Triangle,
                            ),
                        })
                        .collect(),
                },
                Operation::Reverse => Animation {
                    repeat: current.repeat,
                    frames: current.frames.iter().rev().cloned().collect(),
                },
            };
            current = std::sync::Arc::new(next);
        }
        outputs.push(current.clone());
    }
    Ok(outputs)
}

fn dimensions(w: u32, h: u32) -> Result<()> {
    ensure!(
        w > 0 && h > 0 && w <= MAX_DIMENSION && h <= MAX_DIMENSION,
        "Dimensions must be 1–{MAX_DIMENSION}"
    );
    Ok(())
}
