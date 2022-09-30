use std::io::Cursor;
use std::time::Duration;

use egui::{Button, Color32, Context, Frame, Image, Key, Pos2, Rect, Slider, SliderOrientation, Vec2};
use kira::{LoopBehavior, Volume};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::{Easing, Tween};
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, StateEvent, Trans};
use crate::engine::invert_color::{InvertColorCircle, InvertColorRenderer};

pub struct MainMenu {
    win_target: f32,
    bg: Option<egui::TextureHandle>,
    left_color: [f32; 3],
    right_color: [f32; 3],
    vol: f32,
    handle: Option<StaticSoundHandle>,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            win_target: 100.0,
            bg: None,
            left_color: [212.0 / 255.0, 205.0 / 255.0, 241.0 / 255.0],
            right_color: [0.75, 0.0, 0.0],
            vol: 0.5,
            handle: None,
        }
    }
}

impl MainMenu {}

impl GameState for MainMenu {
    fn start(&mut self, s: &mut StateData) {
        if let Some(gpu) = &s.window.gpu {
            s.window.world.insert(InvertColorRenderer::new(gpu));
        }
        if let Some(al) = &mut s.window.audio {
            let music_data = include_bytes!("../../sign/th08_18.mp3");
            let mut s = StaticSoundSettings::default();
            s.loop_behavior = Some(LoopBehavior { start_position: 0.0 });
            let handle = al.manager.play(StaticSoundData::from_cursor(Cursor::new(music_data),
                                                                      s).unwrap())
                .expect("Play bgm failed");
            self.handle = Some(handle);
        }
    }

    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        if s.window.inputs.is_pressed(&[VirtualKeyCode::S]) {
            s.window.inputs.pressed_any_cur_frame = 0;
            (Trans::Push(Box::new(super::ClickState::default())), LoopState::POLL)
        } else {
            (Trans::None, LoopState::POLL)
        }
    }


    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        let mut ret = Trans::None;
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let size = Vec2::new(ui.max_rect().width() / 4.0, ui.max_rect().height() / 4.0);
                ui.allocate_ui_at_rect(Rect {
                    min: Default::default(),
                    max: Pos2::new(ui.max_rect().width(), ui.max_rect().height() - 600.0),
                }, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(size.x);
                        ui.heading("Win Target:");
                        ui.add(Slider::new(&mut self.win_target, 100.0..=1000.0));
                        let mut started = false;
                        if ui.add_sized(size, Button::new("Start")).clicked() {
                            started = true;
                        }
                        ui.heading("BGM Vol:");
                        if let Some(h) = &mut self.handle {
                            if ui.add(Slider::new(&mut self.vol, 0.0..=1.0)).changed() {
                                println!("Changed");
                                h.set_volume(Volume::Amplitude(self.vol as _), Tween {
                                    start_time: Default::default(),
                                    duration: Duration::from_secs(0),
                                    easing: Easing::Linear,
                                }).unwrap();
                            }
                        }
                        if s.window.inputs.is_pressed(&[VirtualKeyCode::Return]) {
                            started = true;
                        }

                        if started {
                            ret = Trans::Push(Box::new(super::MulClickState::new(self.win_target, ui, self.left_color, self.right_color)))
                        }
                    });
                });
                ui.horizontal_centered(|ui| {
                    ui.add_space(size.x);
                    ui.heading("Left Color:");
                    ui.color_edit_button_rgb(&mut self.left_color);
                    ui.add_space(size.x);
                    ui.heading("Right Color:");
                    ui.color_edit_button_rgb(&mut self.right_color);
                })
            });
        ret
    }

    fn shadow_render(&mut self, _: &StateData, ctx: &Context) {
        egui::CentralPanel::default()
            .frame(Frame::none()).show(ctx, |ui| {
            let tex = self.bg.get_or_insert_with(|| {
                ui.ctx().load_texture("bg",
                                      crate::engine::assets::load_image_from_memory(include_bytes!("../../sign/bg.png")).unwrap(),
                                      egui::TextureFilter::Linear)
            });
            let a = 32;
            ui.add(Image::new(tex.id(), ui.max_rect().max.to_vec2())
                .tint(Color32::from_rgb(a, a, a)));
        });
    }
}
