use std::{error::Error, ffi::CStr};

use ash::vk;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Vulkan playground")
        .with_min_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop);

    let entry = ash::Entry::linked();

    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => (),
    });
}

struct VulkanState {
    instance: ash::Instance,
}

impl VulkanState {
    pub fn new(entry: &ash::Entry, window: &Window) -> Result<VulkanState, Box<dyn Error>> {
        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanPlayground\0") };

        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .api_version(vk::make_api_version(0, 1, 1, 0));

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(ash_window::enumerate_required_extensions(window)?);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }?;

        Ok(VulkanState {
            instance,
        })
    }
}

impl Drop for VulkanState {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
