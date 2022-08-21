#![allow(dead_code, unused_imports)]
use futures::executor::block_on;
use imgui::*;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// the actual imnodes samples are in there
mod color_editor;
mod hello_world;
mod multi_editor;

fn main() {
    // Set up window and GPU
    let _event_loop = EventLoop::new();

    // Set up dear imgui
    let mut imgui = imgui::Context::create();
    // Set up dear imnodes
    let _imnodes_ui = imnodes::Context::new();

    let mut _platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
}
