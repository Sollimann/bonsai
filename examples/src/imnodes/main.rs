#![allow(dead_code, unused_imports)]
use futures::executor::block_on;
use glium::glutin;
use glium::{Display, Surface};
use imgui::{self, ChildWindow, CollapsingHeader};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

// the actual imnodes samples are in there
mod color_editor;
mod hello_world;
mod multi_editor;

fn main() {
    let mut imgui = imgui::Context::create();
    let imnodes_ui = imnodes::Context::new();

    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0f32, 768.0f32))
        .with_title("Behavior tree visualizer");

    let context_builder = glutin::ContextBuilder::new();

    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }
    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    let mut _hidpi_factor = platform.hidpi_factor();
    let mut last_frame = Instant::now();
    // let mut last_cursor = None;

    let mut color_editor = color_editor::State::new(&imnodes_ui);

    event_loop.run(move |event, _, control_flow| match event {
        glium::glutin::event::Event::NewEvents(_) => {
            imgui.io_mut().update_delta_time(last_frame.elapsed());
            last_frame = std::time::Instant::now();
        }

        glium::glutin::event::Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            platform
                .prepare_frame(imgui.io_mut(), gl_window.window())
                .expect("Failed to prepare frame");
            gl_window.window().request_redraw();
        }
        glium::glutin::event::Event::WindowEvent {
            event: glium::glutin::event::WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
        }
        glium::glutin::event::Event::RedrawRequested(_) => {
            let now = Instant::now();
            imgui.io_mut().update_delta_time(now - last_frame);
            last_frame = now;
            let ui = imgui.frame();
            color_editor::show(&ui, &mut color_editor);
            let gl_window = display.gl_window();
            let mut target = display.draw();
            target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
            platform.prepare_render(&ui, gl_window.window());
            let draw_data = ui.render();
            renderer.render(&mut target, draw_data).expect("UI rendering failed");
            target.finish().expect("Failed to swap buffers");
        }
        _ => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
        }
    });
}
