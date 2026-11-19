mod cube;
mod timer;

use std::{borrow::Cow, collections::HashSet, time::Instant};

use gpui::{
    App, Application, Bounds, Context, FocusHandle, KeyDownEvent, MouseButton, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};

use cube::{Cube, Face, Move, scramble};
use timer::Timer;

const DEFAULT_FONT_SIZE: u16 = 48;
const FONT_FAMILY: &str = "Oxanium";

struct ScrambledCube {
    moves: [Move; 25],
    cube: Cube,
}

impl ScrambledCube {
    fn new() -> Self {
        let moves = scramble(&mut rand::rng());
        Self {
            cube: Cube::from_moves(&moves),
            moves,
        }
    }
}

struct Cuub {
    focus: FocusHandle,
    timer: Timer,
    puzzle: ScrambledCube,
    font_size: u16,
    pressed: HashSet<String>,
}

impl Cuub {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        window.set_window_title("cuub");
        let focus = cx.focus_handle();
        focus.focus(window);
        cx.observe_window_activation(window, |this, window, _| {
            if !window.is_window_active() {
                this.pressed.clear();
            }
        })
        .detach();
        Self {
            focus,
            timer: Timer::default(),
            puzzle: ScrambledCube::new(),
            font_size: DEFAULT_FONT_SIZE,
            pressed: HashSet::new(),
        }
    }

    fn advance(&mut self, cx: &mut Context<Self>) {
        if self.timer.advance(Instant::now()) {
            self.puzzle = ScrambledCube::new();
        }
        cx.notify();
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        if key.modifiers.control || key.modifiers.platform || key.modifiers.alt {
            return;
        }
        // GPUI's X11 backend can report repeat events with is_held set to false.
        if matches!(key.key.as_str(), "space" | "s" | "f")
            && (event.is_held || !self.pressed.insert(key.key.clone()))
        {
            return;
        }
        match key.key.as_str() {
            "space" => return self.advance(cx),
            "escape" if !event.is_held => return window.remove_window(),
            "s" => self.timer.toggle_scramble(),
            "f" => window.toggle_fullscreen(),
            "0" if !event.is_held => self.font_size = DEFAULT_FONT_SIZE,
            "+" | "=" => self.font_size = (self.font_size + 1).min(400),
            "-" => self.font_size = self.font_size.saturating_sub(1).max(8),
            _ if key.key_char.as_deref() == Some("+") => {
                self.font_size = (self.font_size + 1).min(400);
            }
            _ => return,
        }
        cx.notify();
    }
}

impl Render for Cuub {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.timer.running() {
            window.request_animation_frame();
        }

        let time = self.timer.display(Instant::now());
        let viewport = window.viewport_size();
        let font_size = px(f32::from(self.font_size))
            .min((viewport.width - px(40.0)) / (time.len() as f32 * 0.72))
            .min(viewport.height * 0.32);

        div()
            .size_full()
            .relative()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x121212))
            .text_color(rgb(0xffffff))
            .font_family(FONT_FAMILY)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(|this, event: &gpui::KeyUpEvent, _, _| {
                this.pressed.remove(&event.keystroke.key);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.advance(cx)),
            )
            .child(
                div()
                    .text_size(font_size)
                    .line_height(font_size)
                    .child(time),
            )
            .when(self.timer.show_scramble(), |root| {
                root.child(
                    div()
                        .absolute()
                        .top(px(18.0))
                        .left(px(20.0))
                        .right(px(20.0))
                        .flex()
                        .flex_wrap()
                        .gap_x(px(8.0))
                        .text_size(px(20.0))
                        .line_height(px(24.0))
                        .children(
                            self.puzzle
                                .moves
                                .iter()
                                .map(|turn| div().child(turn.to_string())),
                        ),
                )
                .child(cube_net(&self.puzzle.cube))
            })
    }
}

fn cube_net(cube: &Cube) -> impl IntoElement {
    const STICKER_STEP: f32 = 12.0;
    let faces = [
        (Face::U, 3, 0),
        (Face::L, 0, 3),
        (Face::F, 3, 3),
        (Face::R, 6, 3),
        (Face::B, 9, 3),
        (Face::D, 3, 6),
    ];

    div()
        .absolute()
        .left(px(20.0))
        .bottom(px(24.0))
        .w(px(12.0 * STICKER_STEP))
        .h(px(9.0 * STICKER_STEP))
        .children(faces.into_iter().flat_map(|(face, col, row)| {
            cube.face(face).iter().enumerate().map(move |(i, sticker)| {
                let color = match sticker {
                    Face::U => 0xffffff,
                    Face::R => 0xe62937,
                    Face::F => 0x00e430,
                    Face::D => 0xfdf900,
                    Face::L => 0xffa100,
                    Face::B => 0x0079f1,
                };
                div()
                    .absolute()
                    .left(px((col + i % 3) as f32 * STICKER_STEP))
                    .top(px((row + i / 3) as f32 * STICKER_STEP))
                    .size(px(STICKER_STEP - 2.0))
                    .bg(rgb(color))
            })
        }))
}

fn main() {
    env_logger::init();
    Application::new().run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(include_bytes!("../assets/Oxanium.ttf"))])
            .expect("load Oxanium font");

        let bounds = Bounds::centered(None, size(px(800.0), px(400.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(320.0), px(340.0))),
                app_id: Some("com.github.Coobyk.cuub-rs".into()),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Cuub::new(window, cx)),
        )
        .expect("open cuub window");
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}
