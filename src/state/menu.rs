use egui::{Button, Color32, Context, Frame, Image, ImageData, Key, Pos2, Rect, Slider, SliderOrientation, Vec2};
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, Trans};

pub struct MainMenu {
    win_target: f32,
    bg: Option<egui::TextureHandle>,
    left_color: [f32; 3],
    right_color: [f32; 3],
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            win_target: 100.0,
            bg: None,
            left_color: [0.0, 0.0, 0.75],
            right_color: [0.75, 0.0, 0.0],
        }
    }
}

impl MainMenu {}

impl GameState for MainMenu {
    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        if s.window.inputs.is_pressed(&[VirtualKeyCode::S]) {
            s.window.inputs.pressed_any_cur_frame = 0;
            (Trans::Push(Box::new(super::ClickState::default())), LoopState::POLL)
        } else {
            (Trans::None, LoopState::POLL)
        }
    }


    fn render(&mut self, w: &mut StateData, ctx: &Context) -> Trans {
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
                        if w.window.inputs.is_pressed(&[VirtualKeyCode::Return]) {
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
