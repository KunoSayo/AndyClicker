use std::collections::VecDeque;
use std::default::Default;
use std::time::SystemTime;

use egui::{Color32, Context, Event, Frame, Image, Key, Label, Pos2, Rect, RichText, TextureHandle, Ui};
use egui::TextureFilter::Nearest;
use log::debug;
use specs::WorldExt;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

use crate::engine::{GameState, LoopState, StateData, StateEvent, Trans};
use crate::engine::invert_color::{InvertColorCircle, InvertColorRenderer};

#[derive(Default)]
struct ClickData {
    last_click: Option<SystemTime>,
}

impl ClickData {
    /// Click for now and get the value for click
    /// the value is calculated by 1.0 / dur_s
    pub(crate) fn click(&mut self, now: SystemTime) -> f32 {
        if let Some(last) = &mut self.last_click {
            let dur = now.duration_since(*last).unwrap();
            *last = now;
            1.0 / dur.as_secs_f32()
        } else {
            self.last_click.replace(now);
            0.0
        }
    }
}

pub struct MulClickState {
    start_time: Option<SystemTime>,
    left_click: ClickData,
    right_click: ClickData,
    pressing_a: bool,
    pressing_6: bool,
    left: TextureHandle,
    right: TextureHandle,
    last_time: Option<SystemTime>,
    end_time: Option<SystemTime>,
    /// positive to right
    cur_progress: f32,
    /// positive to right
    a: f32,
    win_target: f32,
    effects: Vec<InvertColorCircle>,
}

impl MulClickState {
    pub(crate) fn new(win_target: f32, ui: &Ui, left_color: [f32; 3], right_color: [f32; 3]) -> Self {
        let to = |x| (x * 255.0) as u8;
        let left = ui.ctx().load_texture("left-color",
                                         egui::ColorImage::new([1, 1],
                                                               Color32::from_rgb(
                                                                   to(left_color[0]), to(left_color[1]), to(left_color[2]))),
                                         Nearest);
        let right = ui.ctx().load_texture("right-color",
                                          egui::ColorImage::new([1, 1],
                                                                Color32::from_rgb(
                                                                    to(right_color[0]), to(right_color[1]), to(right_color[2]))),
                                          Nearest);
        Self {
            start_time: None,
            left_click: Default::default(),
            right_click: Default::default(),
            pressing_a: false,
            win_target,
            left,
            right,

            pressing_6: false,
            cur_progress: 0.0,
            last_time: None,
            a: 0.0,
            end_time: None,
            effects: vec![],
        }
    }

    fn on_event(&mut self, event: &Event, now: SystemTime) {
        if let Event::Key { key, pressed, .. } = event {
            let pressed = *pressed;
            match *key {
                Key::A => {
                    if self.pressing_a {
                        if !pressed {
                            self.pressing_a = false;
                        }
                    } else {
                        if pressed {
                            self.pressing_a = true;
                            self.a += self.left_click.click(now);
                        }
                    }
                }
                Key::Num6 => {
                    if self.pressing_6 {
                        if !pressed {
                            self.pressing_6 = false;
                        }
                    } else {
                        if pressed {
                            self.pressing_6 = true;
                            self.a -= self.right_click.click(now);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl GameState for MulClickState {
    fn start(&mut self, _: &mut StateData) {
        self.start_time.replace(SystemTime::now());
    }

    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        (if s.window.inputs.cur_frame_input.pressing.contains(&VirtualKeyCode::Escape) { Trans::Pop } else { Trans::None }, LoopState::POLL)
    }

    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let now = SystemTime::now();
                let sec = now.duration_since(self.start_time.unwrap()).unwrap().as_secs_f64();
                let max_rect = ui.max_rect();
                if sec > 3.0 {
                    if self.last_time.is_none() {
                        self.last_time.replace(now);
                    }
                    if self.cur_progress.abs() < self.win_target {
                        let now = SystemTime::now();
                        for x in &s.window.egui_ctx.input().events {
                            self.on_event(x, now);
                        }
                    } else {
                        if self.end_time.is_none() {
                            self.end_time = Some(now);
                            self.effects.push(InvertColorCircle {
                                center: [if self.cur_progress > 0.0 { max_rect.max.x } else { 0.0 }, max_rect.height() / 2.0],
                                radius: 0.0,
                            })
                        }
                        for x in &mut self.effects {
                            x.radius += s.dt * 300.0;
                        }
                    }
                    self.cur_progress += s.dt * self.a;
                    let y = ui.max_rect().max.y - 48.0;

                    let mid = (ui.max_rect().max.x / 2.0) * (1.0 + self.cur_progress / self.win_target);
                    let mut right_bottom = Pos2::new(mid, y);
                    let tint = Color32::from_rgba_premultiplied(255, 255, 255, 128);
                    ui.allocate_ui_at_rect(Rect { min: Default::default(), max: right_bottom }, |ui| {
                        ui.add(Image::new(self.left.id(), [mid, y]).tint(tint));
                    });
                    let mid_left = Pos2::new(mid, 0.0);
                    right_bottom.x = ui.max_rect().max.x;
                    ui.allocate_ui_at_rect(Rect { min: mid_left, max: right_bottom }, |ui| {
                        ui.add(Image::new(self.right.id(), [max_rect.max.x - mid, y]).tint(tint));
                    });
                    ui.centered_and_justified(|ui| {
                        ui.heading(format!("{:03.2} ({:.2})", self.cur_progress, self.a));
                    });
                }
                if sec <= 4.0 {
                    let mut r = 255;
                    ui.allocate_ui_at_rect(max_rect, |ui| {
                        ui.centered_and_justified(|ui| {
                            let text = if sec <= 1.0 {
                                format!("Ready {:.02}", 3.0 - sec)
                            } else if sec <= 2.0 {
                                format!("Get   {:.02}", 3.0 - sec)
                            } else if sec <= 3.0 {
                                format!("Set   {:.02}", 3.0 - sec)
                            } else {
                                r = if sec >= 3.5 {
                                    (((4.0 - sec) / 0.5) * 255.0) as u8
                                } else {
                                    255
                                };
                                format!("Go")
                            };
                            ui.add(Label::new(RichText::new(text).color(Color32::from_rgba_premultiplied(255, 255, 255, r)).heading()));
                        });
                    });
                }
            });
        Trans::None
    }

    fn on_event(&mut self, s: Option<&mut StateData>, e: StateEvent) {
        if matches!(e, StateEvent::PostUiRender) {
            let s = s.unwrap();
            if let Some(render) = &s.window.render {
                let renderer = s.window.world.read_resource::<InvertColorRenderer>();
                renderer.render(s.window, &render.views.get_screen().view, &self.effects[..]);
            }
        }
    }
}