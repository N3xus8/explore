use std::sync::Arc;

use winit::{
     window::Window
};


pub struct State {
   pub window: Arc<Window>,
}

impl State {

    
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        Ok(Self {
            window,
        })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {

    }
    
    pub fn render(&mut self) {
        self.window.request_redraw();


    }
}


