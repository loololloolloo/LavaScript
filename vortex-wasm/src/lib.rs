use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VortexRuntime {
    tick: u64,
    connected: bool,
}

#[wasm_bindgen]
impl VortexRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> VortexRuntime {
        VortexRuntime { tick: 0, connected: false }
    }

    pub fn tick(&mut self, dt_ms: f32) {
        if dt_ms.is_finite() && dt_ms >= 0.0 {
            self.tick = self.tick.wrapping_add(1);
        }
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
