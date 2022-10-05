#![allow(dead_code, unused_imports)]
use futures::executor::block_on;
use glium::glutin;
use glium::{Display, Surface};
use imgui::{self, ChildWindow, CollapsingHeader, Condition};
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
        .with_inner_size(glutin::dpi::LogicalSize::new(1280.0f32, 800.0f32))
        .with_title("Behavior tree visualizer");

    let context_builder = glutin::ContextBuilder::new();

    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

    let mut platform = WinitPlatform::init(&mut imgui);
    imgui.set_ini_filename(None);

    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }
    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    let mut hidpi_factor = platform.hidpi_factor();
    let mut width = display.gl_window().window().inner_size().width;
    let mut height = display.gl_window().window().inner_size().height;
    let mut last_frame = Instant::now();

    let mut color_editor = color_editor::State::new(&imnodes_ui);

    event_loop.run(move |event, _, control_flow| match event {
        glutin::event::Event::NewEvents(_) => {
            let now = Instant::now();
            imgui.io_mut().update_delta_time(now - last_frame);
            last_frame = now;
        }
        glutin::event::Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            platform
                .prepare_frame(imgui.io_mut(), gl_window.window())
                .expect("Failed to prepare frame");
            gl_window.window().request_redraw();
        }
        glutin::event::Event::RedrawEventsCleared => {
            let now = Instant::now();
            imgui.io_mut().update_delta_time(now - last_frame);
            last_frame = now;

            let gl_window = display.gl_window();
            platform.prepare_frame(imgui.io_mut(), gl_window.window()).unwrap();
            let ui = imgui.frame();
            // platform.prepare_render(&ui, gl_window.window());

            {
                let window = imgui::Window::new("Hello imnodes")
                    .resizable(true)
                    .position([0.0, 0.0], Condition::Always)
                    .size(
                        [width as f32 / hidpi_factor as f32, height as f32 / hidpi_factor as f32],
                        Condition::Always,
                    );

                window.build(&ui, || {
                    ui.text("Behavior Tree Visualizer");
                    color_editor::show(&ui, &mut color_editor);
                });
            }
            // show imnodes editor

            let mut target = display.draw();

            // the renderer doesn't clear the buffer in case you are displaying imgui widgets
            // over a buffer already containing a game
            target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);

            // ui mowed
            let draw_data = ui.render();
            renderer.render(&mut target, draw_data).expect("UI rendering failed");
            target.finish().expect("Failed to swap buffers");
        }
        glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::Resized(size),
            ..
        } => {
            width = size.width;
            height = size.height;
        }
        glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::ScaleFactorChanged { scale_factor, .. },
            ..
        } => {
            hidpi_factor = scale_factor;
        }
        event => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
        }
    });
}
