use std::collections::HashSet;
use std::mem::swap;

use winit::event::VirtualKeyCode;



#[derive(Debug, Default)]
pub struct RawInputData {
    pub x: f32,
    pub y: f32,
    pub pressing: Box<HashSet<VirtualKeyCode>>,
}


#[derive(Default)]
pub struct BakedInputs {
    pub cur_temp_input: RawInputData,
    pub last_frame_input: RawInputData,
    pub cur_frame_input: RawInputData,
    /// only swap in states.game tick
    pub cur_temp_game_input: RawInputData,
    /// only swap in states.game tick
    pub last_temp_game_input: RawInputData,

    pub pressed_any_cur_frame: bool
}


impl BakedInputs {
    pub fn process(&mut self, pressed: &HashSet<VirtualKeyCode>, released: &HashSet<VirtualKeyCode>) {
        for key in pressed.iter() {
            self.cur_temp_input.pressing.insert(*key);
            self.cur_temp_game_input.pressing.insert(*key);
        }

        for key in released.iter() {
            if self.last_temp_game_input.pressing.contains(key) {
                self.cur_temp_game_input.pressing.remove(key);
            }
            if self.cur_frame_input.pressing.contains(key) {
                self.cur_temp_input.pressing.remove(key);
            }
        }
    }
    /// save current input to last
    /// make current temp input to current frame input
    pub (in crate::engine) fn swap_frame(&mut self) {
        //save current to last
        swap(&mut self.cur_frame_input, &mut self.last_frame_input);
        //clone for not lose temp info
        self.cur_frame_input = self.cur_temp_input.clone();

        self.pressed_any_cur_frame = self.cur_frame_input.pressing.iter().any(|k| !self.last_frame_input.pressing.contains(k));

    }

    #[allow(unused)]
    pub fn is_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|k| !self.last_frame_input.pressing.contains(k))
            && keys.iter().all(|k| self.cur_frame_input.pressing.contains(k))
    }
}

impl RawInputData {
    #[allow(unused)]
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Clone for RawInputData {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            pressing: self.pressing.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
        self.pressing = source.pressing.clone();
    }
}