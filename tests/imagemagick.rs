use image::{Rgba, RgbaImage};
use img::{Animation, Card, Frame, Operation};
#[test]
fn optional_imagemagick_negates_rgb_and_preserves_timing() {
    if std::process::Command::new("magick")
        .arg("-version")
        .output()
        .is_err()
    {
        eprintln!("ImageMagick not installed; optional integration skipped");
        return;
    }
    let source = std::sync::Arc::new(Animation {
        frames: vec![Frame {
            pixels: RgbaImage::from_pixel(3, 2, Rgba([255, 0, 0, 255])),
            delay_ms: 70,
        }],
        repeat: gif::Repeat::Finite(2),
    });
    let outputs = img::process(
        source,
        &[Card {
            id: 1,
            enabled: true,
            operation: Operation::ImageMagickNegate,
        }],
    )
    .unwrap();
    assert_eq!(
        outputs[0].frames[0].pixels.get_pixel(0, 0),
        &Rgba([0, 255, 255, 255])
    );
    assert_eq!(outputs[0].frames[0].delay_ms, 70);
    assert_eq!(outputs[0].repeat, gif::Repeat::Finite(2));
}
