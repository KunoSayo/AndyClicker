use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::time::Duration;

use alto::Source;
use egui::{Button, Color32, Context, Frame, Image, Key, Pos2, Rect, Slider, SliderOrientation, Vec2};
use specs::WorldExt;
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, StateEvent, Trans};
use crate::engine::invert_color::{InvertColorCircle, InvertColorRenderer};

pub struct MainMenu {
    win_target: f32,
    bg: Option<egui::TextureHandle>,
    left_color: [f32; 3],
    right_color: [f32; 3],
    gain: f32,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            win_target: 100.0,
            bg: None,
            left_color: [212.0 / 255.0, 205.0 / 255.0, 241.0 / 255.0],
            right_color: [0.75, 0.0, 0.0],
            gain: 1.0,
        }
    }
}

impl MainMenu {}

impl GameState for MainMenu {
    fn start(&mut self, s: &mut StateData) {
        if let Some(gpu) = &s.window.gpu {
            s.window.world.insert(InvertColorRenderer::new(gpu));
        }
        if let Some(al) = &mut s.window.al {
            self.gain = al.bgm_source.gain();
            let music_data = include_bytes!("../../sign/th08_18.mp3");
            let (audio_bin, freq, channel) = {
                let mut decoder = minimp3::Decoder::new(&music_data[..]);
                let mut fst = match decoder.next_frame() {
                    Ok(f) => f,
                    Err(e) => {
                        log::error!("Decode mp3 file failed for {:?}", e);
                        panic!("Decoder mp3 file first audio frame failed for {:?}", e);
                    }
                };
                let freq = fst.sample_rate;
                let channel = fst.channels;
                let mut audio_bin = Vec::with_capacity(8 * 1024 * 1024);
                audio_bin.append(&mut fst.data);
                while let Ok(mut frame) = decoder.next_frame() {
                    debug_assert!(frame.channels == channel);
                    debug_assert!(frame.sample_rate == freq);
                    audio_bin.append(&mut frame.data);
                }
                audio_bin.resize(audio_bin.len(), 0);
                (audio_bin, freq, channel)
            };
            log::info!("Loaded bgm and it has {} channels", channel);

            let buf = if channel == 1 {
                Arc::new(al.ctx.new_buffer::<alto::Mono<i16>, _>(&audio_bin, freq).unwrap())
            } else {
                Arc::new(al.ctx.new_buffer::<alto::Stereo<i16>, _>(&audio_bin, freq).unwrap())
            };
            al.play_bgm(buf);
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
                        if let Some(al) = &mut s.window.al {
                            ui.add(Slider::new(&mut self.gain, al.bgm_source.min_gain()..=al.bgm_source.max_gain()));
                            al.bgm_source.set_gain(self.gain).unwrap();
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
