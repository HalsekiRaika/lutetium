pub struct RunningState(State);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum State {
    Active,
    Shutdown
}

impl RunningState {
    pub(crate) fn switch(&mut self, mut f: impl FnMut(&mut State)) {
        f(&mut self.0)
    }
    
    pub fn is_active(&self) -> bool {
        if let State::Active = self.0 {
            return true;
        }
        false
    }
    
    pub fn available_shutdown(&self) -> bool {
        !self.is_active()
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self(State::Active)
    }
}