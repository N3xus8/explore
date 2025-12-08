use winit::{
     event_loop::EventLoop, 
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;

use spinoff::app::App;

fn main() {
   run().expect("failed to run wgpu")
}



pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}