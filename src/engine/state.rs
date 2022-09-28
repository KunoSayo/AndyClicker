use std::time::Duration;

use mlua::UserData;
use winit::event_loop::ControlFlow;

use crate::engine::app::WindowInstance;

#[allow(unused)]
pub enum Trans {
    None,
    Push(Box<dyn GameState>),
    Pop,
    Switch(Box<dyn GameState>),
    Exit,
    Vec(Vec<Trans>),
}

impl Default for Trans {
    fn default() -> Self {
        Self::None
    }
}

pub struct StateData<'a> {
    pub window: &'a mut WindowInstance,
    pub dt: f32,
}


pub trait GameState: 'static {
    fn start(&mut self, _: &mut StateData) {}

    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) { (Trans::None, LoopState::WAIT) }

    fn shadow_update(&mut self) -> LoopState { LoopState::WAIT_ALL }

    fn render(&mut self, _: &mut StateData, _: &egui::Context) -> Trans { Trans::None }

    fn shadow_render(&mut self, _: &StateData, _: &egui::Context) {}

    fn stop(&mut self, _: &mut StateData) {}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct LoopState {
    pub control_flow: ControlFlow,
    pub render: bool,
}

impl UserData for LoopState {}


impl LoopState {
    #[allow(unused)]
    pub const WAIT_ALL: LoopState = LoopState {
        control_flow: ControlFlow::Wait,
        render: false,
    };

    #[allow(unused)]
    pub const WAIT: LoopState = LoopState {
        control_flow: ControlFlow::Wait,
        render: true,
    };

    #[allow(unused)]
    pub const POLL: LoopState = LoopState {
        control_flow: ControlFlow::Poll,
        render: true,
    };

    #[allow(unused)]
    pub const POLL_WITHOUT_RENDER: LoopState = LoopState {
        control_flow: ControlFlow::Poll,
        render: false,
    };

    #[allow(unused)]
    pub fn wait_until(dur: Duration, render: bool) -> Self {
        Self {
            control_flow: ControlFlow::WaitUntil(std::time::Instant::now() + dur),
            render,
        }
    }
}

impl GameState for () {}

impl std::ops::BitOrAssign for LoopState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.render |= rhs.render;
        if self.control_flow != rhs.control_flow {
            match self.control_flow {
                ControlFlow::Wait => self.control_flow = rhs.control_flow,
                ControlFlow::WaitUntil(t1) => match rhs.control_flow {
                    ControlFlow::Wait => {}
                    ControlFlow::WaitUntil(t2) => {
                        self.control_flow = ControlFlow::WaitUntil(t1.min(t2));
                    }
                    _ => {
                        self.control_flow = rhs.control_flow;
                    }
                },
                _ => {}
            }
        }
    }
}

