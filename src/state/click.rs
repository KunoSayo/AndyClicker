use std::time::SystemTime;
use egui::{Button, Context, Frame, Label, Pos2, Rect, Stroke, Vec2, WidgetText};
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
            clicks: vec![now]
        }
    }
}

#[derive(Default)]
pub struct MainState {
    click: Option<ClickData>
}

impl MainState {
    fn click(&mut self) {
        let now = SystemTime::now();
        if let Some(click) = &mut self.click {
            click.clicks.push(now);
        } else {
            self.click = Some(ClickData::click_first());
        }
    }
}

impl GameState for MainState {
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
                    ui.set_min_size(bs);
                    ui.add_enabled_ui(self.click.is_some(), |ui| {
                        if ui.add_sized(bs, Button::new("Reset")).clicked() {
                            self.click = None;
                        }
                    });
                    bs.y *= 2.0;
                    let mut clicked = false;
                    if ui.add_sized(bs, Button::new("Click")).clicked() {
                        clicked = true;
                    }
                    clicked |= s.window.inputs.pressed_any_cur_frame;
                    if clicked {
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
                            ui.label(format!("CPS: {:.2} BPM: {:.2}", cps, bpm));
                            ui.label(format!("MAX: CPS: {:.2} BPM: {:.2}", click.max_cps, max_bpm));
                        });

                    }
                });
            });
        Trans::None
    }
}