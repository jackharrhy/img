use img::{Animation, Card, Frame, Operation, process};
use std::sync::Arc;

#[test]
fn stack_resizes_reverses_bypasses_and_reorders() {
    let source = Arc::new(Animation {
        frames: vec![
            Frame {
                pixels: RgbaImage::from_pixel(4, 3, Rgba([255, 0, 0, 255])),
                delay_ms: 70,
            },
            Frame {
                pixels: RgbaImage::from_pixel(4, 3, Rgba([0, 255, 0, 255])),
                delay_ms: 130,
            },
        ],
        repeat: gif::Repeat::Infinite,
    });
    let mut cards = vec![
        Card {
            id: 1,
            enabled: true,
            operation: Operation::Resize {
                width: 2,
                height: 2,
            },
        },
        Card {
            id: 2,
            enabled: true,
            operation: Operation::Reverse,
        },
    ];
    let out = process(source.clone(), &cards).unwrap();
    assert_eq!(out[1].frames[0].pixels.dimensions(), (2, 2));
    assert_eq!(out[1].frames[0].delay_ms, 130);
    assert_eq!(
        out[1].frames[0].pixels.get_pixel(0, 0),
        &Rgba([0, 255, 0, 255])
    );
    cards[0].enabled = false;
    assert_eq!(
        process(source.clone(), &cards).unwrap()[1].frames[0]
            .pixels
            .dimensions(),
        (4, 3)
    );
    cards.swap(0, 1);
    cards[1].enabled = true;
    assert_eq!(
        process(source.clone(), &cards).unwrap()[0].frames[0]
            .pixels
            .dimensions(),
        (4, 3)
    );
    cards.remove(0);
    assert_eq!(process(source, &cards).unwrap()[0].frames[0].delay_ms, 70);
}
use image::{Rgba, RgbaImage};

#[test]
fn gif_roundtrip_preserves_pixels_delays_and_loop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.gif");
    let source = Animation {
        frames: vec![
            Frame {
                pixels: RgbaImage::from_pixel(4, 3, Rgba([255, 0, 0, 255])),
                delay_ms: 70,
            },
            Frame {
                pixels: RgbaImage::from_pixel(4, 3, Rgba([0, 0, 0, 0])),
                delay_ms: 130,
            },
        ],
        repeat: gif::Repeat::Finite(3),
    };
    source.save(&path).unwrap();
    let decoded = Animation::load(&path).unwrap();
    assert_eq!(decoded.repeat, source.repeat);
    assert_eq!(decoded.frames.len(), 2);
    for (a, b) in source.frames.iter().zip(&decoded.frames) {
        assert_eq!(a.delay_ms, b.delay_ms);
        assert_eq!(a.pixels, b.pixels);
    }
}
