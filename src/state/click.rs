use std::time::SystemTime;

use egui::{Button, Context, Frame, Pos2, Rect, Vec2};

use crate::engine::{GameState, LoopState, StateData, Trans};

struct ClickData {
    max_cps: f64,
    clicks: Vec<SystemTime>,
}

impl ClickData {
    fn click_first() -> ClickData {
        let now = SystemTime::now();
        ClickData {
            max_cps: 0.0,
            clicks: vec![now],
        }
    }
}

#[derive(Default)]
pub struct ClickState {
    click: Option<ClickData>,
}

impl ClickState {
    fn click(&mut self) {
        let now = SystemTime::now();
        if let Some(click) = &mut self.click {
            click.clicks.push(now);
        } else {
            self.click = Some(ClickData::click_first());
        }
    }
}

impl GameState for ClickState {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::POLL)
    }

    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let now = SystemTime::now();
                ui.allocate_ui_at_rect(Rect {
                    min: Pos2::new(0.0, 0.0),
                    max: ui.max_rect().max,
                }, |ui| {
                    let mut bs = Vec2::new(ui.max_rect().width(), ui.max_rect().height() / 4.0);
                    ui.add_enabled_ui(self.click.is_some(), |ui| {
                        if ui.add_sized(bs, Button::new("Reset")).clicked() {
                            self.click = None;
                        }
                    });
                    let delta = bs.x;
                    ui.horizontal(|ui| {
                        if ui.add_sized(bs, Button::new("Click")).clicked() {
                            self.click();
                        }
                    });

                    for _ in 0..s.window.inputs.pressed_any_cur_frame {
                        self.click();
                    }

                    if let Some(click) = &mut self.click {
                        let all = click.clicks.len();
                        let start = click.clicks[0];
                        let dur = now.duration_since(start).map_err(|x| x.duration()).unwrap_or_else(|x| x);
                        let sec = dur.as_secs_f64();
                        let cps = all as f64 / sec;
                        let bpm = 15.0 * cps;
                        if sec >= 1.0 {
                            click.max_cps = cps.max(click.max_cps);
                        }
                        let max_bpm = 15.0 * click.max_cps;
                        ui.vertical_centered(|ui| {
                            ui.label(format!("Clicks: {}", all));
                            ui.label(format!("CPS: {:.2} BPM: {:.2}", cps, bpm));
                            ui.label(format!("MAX: CPS: {:.2} BPM: {:.2}", click.max_cps, max_bpm));
                        });
                    }
                });
            });
        Trans::None
    }
}