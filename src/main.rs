use std::{
    error::Error,
    ffi::{c_void, CStr},
};

use ash::{
    extensions::{ext::DebugUtils, khr},
    vk,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Vulkan playground")
        .with_min_inner_size(PhysicalSize::<u32>::new(1280, 720))
        .build(&event_loop)?;

    let entry = ash::Entry::linked();

    let state = VulkanState::new(&entry, &window)?;

    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            println!("Window closed");
            *control_flow = ControlFlow::Exit;
        }
        _ => (),
    });
}

struct InstanceState {
    instance: ash::Instance,
    debug_utils: DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl InstanceState {
    pub fn new(entry: &ash::Entry, window: &Window) -> Result<InstanceState, Box<dyn Error>> {
        let validation_layer_name =
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };
        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanPlayground\0") };

        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .api_version(vk::make_api_version(0, 1, 1, 0));

        let layers = [validation_layer_name.as_ptr() as _];

        let mut extensions = vec![DebugUtils::name().as_ptr()];
        extensions.extend(ash_window::enumerate_required_extensions(window)?);

        let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions[..])
            .push_next(&mut debug_create_info);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }?;
        let debug_utils = DebugUtils::new(entry, &instance);

        let mut state = InstanceState {
            instance,
            debug_utils,
            debug_messenger: vk::DebugUtilsMessengerEXT::null(),
        };

        let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &state.instance);
        state.debug_messenger =
            unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None) }?;

        Ok(state)
    }
}

impl Drop for InstanceState {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

struct VulkanState {
    surface: vk::SurfaceKHR,
    surface_fns: khr::Surface,
    // Drop order of instance is intentional
    inst: InstanceState,
}

impl VulkanState {
    pub fn new(entry: &ash::Entry, window: &Window) -> Result<VulkanState, Box<dyn Error>> {
        let inst = InstanceState::new(entry, window)?;
        let surface_fns = khr::Surface::new(entry, &inst.instance);

        let mut state = VulkanState {
            surface: vk::SurfaceKHR::null(),
            surface_fns,
            inst,
        };

        state.surface =
            unsafe { ash_window::create_surface(entry, &state.inst.instance, window, None) }?;

        // Find device which supports the surface
        let physical_devices = unsafe { state.inst.instance.enumerate_physical_devices() }?;

        // TODO: Replace with function to check if a device is valid.
        let phy = physical_devices
            .iter()
            .find(|phy| {
                let properties =
                    unsafe { state.inst.instance.get_physical_device_properties(**phy) };
                let queue_properties = unsafe {
                    state
                        .inst
                        .instance
                        .get_physical_device_queue_family_properties(**phy)
                };
                unsafe {
                    state.surface_fns.get_physical_device_surface_support(
                        **phy,
                        todo!(),
                        state.surface,
                    )
                }
                .ok()
                .unwrap_or(false)
            })
            .cloned()
            .expect("no device");

        Ok(state)
    }
}

impl Drop for VulkanState {
    fn drop(&mut self) {
        unsafe {
            self.surface_fns.destroy_surface(self.surface, None);
        }

        // InstanceState is dropped in it's own drop impl
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}
