use super::*;

fn frame(ctx: &egui::Context, app: &mut Editor, events: Vec<egui::Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(980., 820.),
            )),
            events,
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    )
}

fn text_center(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Pos2 {
    shapes
        .iter()
        .rev()
        .find_map(|shape| {
            if let egui::Shape::Text(shape) = &shape.shape
                && shape.galley.text() == text
                && (!matches!(text, "Import GIF" | "Resize" | "Reverse") || shape.pos.x < 360.)
            {
                Some(shape.pos + shape.galley.size() * 0.5)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("Missing UI text: {text}"))
}

fn click(ctx: &egui::Context, app: &mut Editor, text: &str) {
    let output = frame(ctx, app, vec![]);
    let pos = text_center(&output.shapes, text);
    click_at(ctx, app, pos);
}

fn click_at(ctx: &egui::Context, app: &mut Editor, pos: egui::Pos2) {
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                },
            ],
        );
    }
    settle(ctx, app);
}

fn visibility_center(ctx: &egui::Context, id: u64) -> egui::Pos2 {
    ctx.read_response(egui::Id::new(("step-action", id, "visibility")))
        .unwrap()
        .rect
        .center()
}

fn settle(ctx: &egui::Context, app: &mut Editor) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.receiver.is_some() {
        assert!(Instant::now() < deadline, "UI processing timed out");
        std::thread::sleep(Duration::from_millis(5));
        frame(ctx, app, vec![]);
    }
}

#[test]
fn operation_menu_reorder_bypass_and_remove() {
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        let ctx = egui::Context::default();
        ctx.set_theme(theme);
        let mut app = Editor::default();
        frame(&ctx, &mut app, vec![]);
        let empty = frame(&ctx, &mut app, vec![]);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        text_center(&empty.shapes, "Choose a GIF to preview.");
        app.source = Some(Arc::new(crate::sample()));
        app.name = "sample.gif".into();
        assert!(
            app.cards.is_empty(),
            "Import must start without preset edits"
        );

        click(&ctx, &mut app, "Add operation");
        click(&ctx, &mut app, "Resize");
        assert_eq!(app.cards.len(), 1);
        assert_eq!(app.outputs[0].frames[0].pixels.dimensions(), (320, 240));
        assert_eq!(app.selected_id(), app.cards[0].id);
        frame(&ctx, &mut app, vec![]);
        assert_eq!(app.preview.texture.as_ref().unwrap().0, app.cards[0].id);
        click(&ctx, &mut app, "Preview");
        assert_eq!(app.selected_id(), 0);

        click(&ctx, &mut app, "Add operation");
        click(&ctx, &mut app, "Reverse");
        assert_eq!(app.cards.len(), 2);
        assert_eq!(
            app.selected_id(),
            app.cards[1].id,
            "Adding a step selects the latest output"
        );
        click(&ctx, &mut app, "Preview");
        let pinned = app.cards[0].id;
        assert_eq!(app.selected_id(), pinned);
        let pos = ctx
            .read_response(egui::Id::new(("step-action", app.cards[1].id, "up")))
            .unwrap()
            .rect
            .center();
        click_at(&ctx, &mut app, pos);
        assert_eq!(
            app.selected_id(),
            pinned,
            "Preview follows the selected step when reordered"
        );
        assert!(matches!(app.cards[0].operation, Operation::Reverse));

        frame(&ctx, &mut app, vec![]);
        let pos = visibility_center(&ctx, app.cards[1].id);
        click_at(&ctx, &mut app, pos);
        assert!(!app.cards[1].enabled);
        assert_eq!(app.outputs[1].frames[0].pixels.dimensions(), (240, 180));
        let pos = ctx
            .read_response(egui::Id::new(("step-action", pinned, "delete")))
            .unwrap()
            .rect
            .center();
        click_at(&ctx, &mut app, pos);
        assert_eq!(
            app.selected_id(),
            app.cards[0].id,
            "Removing the selected step returns to latest"
        );
        let pos = ctx
            .read_response(egui::Id::new(("step-action", app.cards[0].id, "delete")))
            .unwrap()
            .rect
            .center();
        click_at(&ctx, &mut app, pos);
        assert!(app.cards.is_empty());
        assert!(app.outputs.is_empty());
    }
}

#[test]
fn dragging_inserts_at_the_indicated_gap() {
    let ctx = egui::Context::default();
    let mut app = Editor {
        source: Some(Arc::new(crate::sample())),
        ..Default::default()
    };
    frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, "Add operation");
    click(&ctx, &mut app, "Resize");
    click(&ctx, &mut app, "Add operation");
    click(&ctx, &mut app, "Reverse");
    click(&ctx, &mut app, "Preview");
    let pinned = app.selected_id();
    frame(&ctx, &mut app, vec![]);
    let original = frame(&ctx, &mut app, vec![]);
    let original_width = text_center(&original.shapes, "Width");
    let start = ctx
        .read_response(egui::Id::new(("drag", app.cards[0].id)))
        .unwrap()
        .rect
        .center();
    let end = ctx
        .read_response(egui::Id::new(("step-action", app.cards[1].id, "delete")))
        .unwrap()
        .rect
        .center()
        + egui::vec2(0., 30.);
    for outcome in ["cancel", "outside", "drop"] {
        let target = if outcome == "outside" {
            egui::pos2(700., 500.)
        } else {
            end
        };
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(0., 15.))],
        );
        let dragging = frame(&ctx, &mut app, vec![egui::Event::PointerMoved(target)]);
        assert!(egui::DragAndDrop::has_payload_of_type::<u64>(&ctx));
        let floating_width = text_center(&dragging.shapes, "Width");
        assert!(
            (floating_width - original_width - (target - start)).length() < 1.,
            "The actual card contents must follow the pointer without changing the grab offset"
        );
        let width_labels = dragging
            .shapes
            .iter()
            .filter(|shape| {
                matches!(&shape.shape,
            egui::Shape::Text(text) if text.galley.text() == "Width")
            })
            .count();
        assert_eq!(
            width_labels, 1,
            "The original slot must not contain a duplicate card"
        );
        assert!(!dragging.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Text(text) if text.galley.text().starts_with("Move Resize"))));
        let indicator = dragging.shapes.iter().any(|s| {
            matches!(&s.shape,
            egui::Shape::LineSegment { stroke, .. } if stroke.width == 2.)
        });
        assert_eq!(indicator, outcome != "outside");
        if outcome == "cancel" {
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            );
            assert!(!egui::DragAndDrop::has_payload_of_type::<u64>(&ctx));
        }
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        if outcome != "drop" {
            assert!(matches!(app.cards[0].operation, Operation::Resize { .. }));
        }
    }
    settle(&ctx, &mut app);
    assert!(matches!(app.cards[0].operation, Operation::Reverse));
    assert_eq!(app.cards[1].id, pinned);
    assert_eq!(app.selected_id(), pinned);
    assert_eq!(app.preview.texture.as_ref().unwrap().0, pinned);
}

#[test]
fn frame_strip_seeks_and_playback_resumes_from_the_selected_frame() {
    let ctx = egui::Context::default();
    let mut app = Editor {
        source: Some(Arc::new(crate::sample())),
        ..Default::default()
    };
    frame(&ctx, &mut app, vec![]);
    assert!(!app.preview.frames_open);
    click(&ctx, &mut app, "Show frames");
    assert!(app.preview.frames_open);
    click(&ctx, &mut app, "3   80 ms");
    assert!(app.preview.paused);
    assert_eq!(app.preview.frame_index(app.source.as_ref().unwrap()), 2);
    click(&ctx, &mut app, "›");
    assert_eq!(app.preview.frame_index(app.source.as_ref().unwrap()), 3);
    click(&ctx, &mut app, "‹");
    assert_eq!(app.preview.frame_index(app.source.as_ref().unwrap()), 2);
    click(&ctx, &mut app, "Play");
    assert!(!app.preview.paused);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(app.preview.paused);
    click(&ctx, &mut app, "Hide frames");
    assert!(!app.preview.frames_open);
}

#[test]
fn processing_keeps_the_layout_and_last_preview() {
    let ctx = egui::Context::default();
    let source = Arc::new(crate::sample());
    let mut app = Editor {
        source: Some(source),
        ..Default::default()
    };
    frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, "Add operation");
    click(&ctx, &mut app, "Resize");
    app.preview.frames_open = true;
    let ready = frame(&ctx, &mut app, vec![]);
    let anchors = ["Import GIF", "Resize", "Export result…", "Hide frames"];
    let positions: Vec<_> = anchors
        .iter()
        .map(|text| text_center(&ready.shapes, text))
        .collect();
    let previous = app.last_preview.as_ref().unwrap().1.clone();
    let pos = visibility_center(&ctx, app.cards[0].id);
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    let changed = frame(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    assert!(!app.cards[0].enabled);
    assert_eq!(
        text_center(&changed.shapes, "Hide frames"),
        text_center(&ready.shapes, "Hide frames")
    );
    let (tx, rx) = mpsc::channel();
    app.receiver = Some(rx);
    let loading = frame(&ctx, &mut app, vec![]);
    for (text, position) in anchors.iter().zip(positions) {
        assert_eq!(
            text_center(&loading.shapes, text),
            position,
            "{text} moved during processing"
        );
    }
    assert!(Arc::ptr_eq(
        &app.last_preview.as_ref().unwrap().1,
        &previous
    ));
    assert_eq!(app.preview.texture.as_ref().unwrap().0, app.cards[0].id);
    tx.send(Ok(Completed::Processed(
        img::process(app.source.clone().unwrap(), &app.cards).unwrap(),
    )))
    .unwrap();
    let complete = frame(&ctx, &mut app, vec![]);
    assert_eq!(
        text_center(&complete.shapes, "Resize"),
        text_center(&ready.shapes, "Resize")
    );
    assert_eq!(
        text_center(&complete.shapes, "Hide frames"),
        text_center(&ready.shapes, "Hide frames")
    );
}

#[test]
fn negate_from_menu_processes_real_gif_and_can_be_bypassed() {
    let ctx = egui::Context::default();
    let source = Arc::new(
        Animation::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hummingbird.gif"),
        )
        .unwrap(),
    );
    let mut app = Editor {
        source: Some(source.clone()),
        ..Default::default()
    };
    frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, "Add operation");
    click(&ctx, &mut app, "Negate");
    assert!(app.status.is_empty(), "Processing failed: {}", app.status);
    assert!(matches!(app.cards[0].operation, Operation::Negate));
    assert_eq!(app.selected_id(), app.cards[0].id);
    assert_eq!(app.outputs[0].frames.len(), source.frames.len());
    let original = source.frames[0].pixels.get_pixel(100, 100).0;
    assert_eq!(
        app.outputs[0].frames[0].pixels.get_pixel(100, 100).0,
        [
            255 - original[0],
            255 - original[1],
            255 - original[2],
            original[3]
        ]
    );
    let pos = visibility_center(&ctx, app.cards[0].id);
    click_at(&ctx, &mut app, pos);
    assert!(Arc::ptr_eq(&app.outputs[0], &source));
    click_at(&ctx, &mut app, pos);
    assert!(app.status.is_empty(), "Processing failed: {}", app.status);
    assert_ne!(app.outputs[0].frames[0].pixels, source.frames[0].pixels);
}
