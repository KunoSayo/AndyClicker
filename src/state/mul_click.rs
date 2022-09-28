use std::collections::VecDeque;
use std::default::Default;
use std::fmt::format;
use std::time::SystemTime;

use egui::{Color32, Context, Event, Frame, Image, Key, Label, Pos2, Rect, RichText, TextureHandle, Ui, Vec2};
use egui::TextureFilter::Nearest;
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, Trans};

#[derive(Default)]
struct ClickData {
    clicks: VecDeque<SystemTime>,
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
    /// positive to right
    cur_progress: f32,
    /// positive to right
    a: f32,
    win_target: f32,
}


impl MulClickState {
    /// click and get cps
    fn click(click: &mut ClickData) {
        let now = SystemTime::now();
        click.clicks.push_back(now);
    }

    fn get_cps(left: &SystemTime, now: &SystemTime, click: &mut ClickData) -> f32 {
        while let Some(front) = click.clicks.front() {
            if left.duration_since(*front).is_ok() {
                click.clicks.pop_front();
            } else {
                break;
            }
        }
        let sec = now.duration_since(*left).unwrap().as_secs_f32();
        click.clicks.len() as f32 / sec
    }


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
                let a_times = get_key_press_times(ui, Key::A, &mut self.pressing_a);
                let six_times = get_key_press_times(ui, Key::Num6, &mut self.pressing_6);
                let max_rect = ui.max_rect();
                if sec > 3.0 {
                    if self.last_time.is_none() {
                        self.last_time.replace(now);
                    }
                    for _ in 0..a_times {
                        Self::click(&mut self.left_click);
                    }
                    for _ in 0..six_times {
                        Self::click(&mut self.right_click);
                    }
                    if let Ok(dur) = now.duration_since(self.last_time.unwrap()) {
                        let dur = dur.as_secs_f32();
                        if dur >= 0.25 {
                            let left_cps = Self::get_cps(self.last_time.as_ref().unwrap(), &now, &mut self.left_click);
                            let right_cps = Self::get_cps(self.last_time.as_ref().unwrap(), &now, &mut self.right_click);
                            self.a += left_cps - right_cps;
                            self.last_time.replace(now);
                        }
                        self.cur_progress += s.dt * self.a;
                        let y = ui.max_rect().max.y;
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
                    }
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
                            ui.add(Label::new(RichText::new(text).color(Color32::from_rgba_premultiplied(255, 0, 9, r)).heading()));
                        });
                    });
                }
            });
        Trans::None
    }
}

fn get_key_press_times(ui: &Ui, k: Key, last: &mut bool) -> (usize) {
    let mut ret = 0;
    for x in &ui.input().events {
        match x {
            Event::Key { key, pressed, .. } if key == &k => {
                if *pressed {
                    if !*last {
                        ret += 1;
                        *last = true;
                    }
                } else {
                    *last = false;
                }
            }
            _ => {}
        }
    }
    ret
}