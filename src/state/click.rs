use egui::{Context, Frame, Pos2, Rect};
use crate::engine::{GameState, LoopState, StateData, Trans};

#[derive(Default)]
pub struct MainState {
    age: i32,
    name: String,
}

impl GameState for MainState {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::POLL)
    }

    fn render(&mut self, _: &mut StateData, ctx: &Context) -> Trans {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.allocate_ui_at_rect(Rect {
                    min: Pos2::new(0.0, 0.0),
                    max: ui.max_rect().max,
                }, |ui| {
                    ui.heading("My egui Application");
                    ui.horizontal(|ui| {
                        ui.label("Your name: ");
                        ui.text_edit_singleline(&mut self.name);
                    });
                    ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                    if ui.button("Click each year").clicked() {
                        self.age += 1;
                    }
                    ui.label(format!("Hello '{}', age {}", self.name, self.age));
                });
            });
        Trans::None
    }
}