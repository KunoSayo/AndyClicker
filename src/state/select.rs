use std::time::SystemTime;

use egui::{Button, Color32, Context, Frame, Image, ImageData, Pos2, Rect, Slider, SliderOrientation, Vec2};
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, Trans};

pub struct MainMenu {
    win_target: f64,
    bg: Option<egui::TextureHandle>,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            win_target: 1.0,
            bg: None,
        }
    }
}

impl MainMenu {}

impl GameState for MainMenu {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::WAIT)
    }


    fn render(&mut self, w: &mut StateData, ctx: &Context) -> Trans {
        let mut ret = Trans::None;
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let size = Vec2::new(ui.max_rect().width() / 4.0, ui.max_rect().height() / 4.0);
                    ui.add_space(size.x);
                    ui.heading("Win Target:");
                    ui.add(Slider::new(&mut self.win_target, 1.0..=100.0));
                    let mut started = false;
                    if ui.add_sized(size, Button::new("Start")).clicked() {
                        started = true;
                    }
                    if w.window.inputs.is_pressed(&[VirtualKeyCode::Return]) {
                        started = true;
                    }
                    if started {
                        ret = Trans::Push(Box::new(super::ClickState::default()))
                    }
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
