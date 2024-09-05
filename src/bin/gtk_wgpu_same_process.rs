use std::time::Duration;

use dma_buf::egl_functions;
use dma_buf::texture_export_wgpu;
use dma_buf::wgpu_context::WgpuContext;
use gtk4::{gdk, glib, prelude::*, subclass::prelude::*};

/// This example creates a WGPU OpenGL texture, exports it via DMA-BUF
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    // create the application
    let application = gtk4::Application::builder().application_id("hello").build();
    application.connect_activate(|app| {
        // Render to texture
        let wgpu_context = pollster::block_on(WgpuContext::create());
        wgpu_context.render_to_texture();
        let texture = texture_export_wgpu::export_to_opengl_texture(&wgpu_context.texture)
            .expect("texture not created");
        let (texture_storage_meta_data, dma_buf_fd) =
            texture_export_wgpu::export_to_dma_buf(&wgpu_context.adapter, texture);

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Triangle")
            .default_height(600)
            .default_width(600)
            .build();

        // get the display
        // need to call dmabuf_formats for the graphics offload part to work (and get initialized)
        let dma_buf_formats = gdk::Display::default().unwrap().dmabuf_formats();
        println!("supported dma buf formats : {:?}", dma_buf_formats);

        let rows: gtk4::Box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let child = gtk4::GLArea::new();
        let dmabuf_widget = gtk4::GraphicsOffload::new(Some(&child));
        dmabuf_widget.set_enabled(gtk4::GraphicsOffloadEnabled::Enabled);

        // create the texture
        let texture_gtk = gdk::DmabufTextureBuilder::new();
        texture_gtk.set_fourcc(texture_storage_meta_data.fourcc as u32);
        texture_gtk.set_offset(0, texture_storage_meta_data.offset as u32); //right plane ?
        texture_gtk.set_stride(0, texture_storage_meta_data.stride as u32); //right plane ?
        texture_gtk.set_modifier(texture_storage_meta_data.modifiers);
        texture_gtk.set_fd(0, dma_buf_fd);
        texture_gtk.set_height(egl_functions::EGL_HEIGHT);
        texture_gtk.set_width(egl_functions::EGL_WIDTH);
        unsafe {
            let texture = texture_gtk.build().unwrap();
        }

        rows.append(&dmabuf_widget);
        window.set_visible(true);

        // rendering for the gl area
            std::thread::spawn(move || {
            wgpu_context.animate(); // we should pass some other things to call to gtk for a texture change
        });
    });

    application.run();
}

// create a custom widget that has the paintable interface
// mod imp {
//     use super::*;

//     #[derive(Debug)]
//     pub(crate) struct WgpuPaintable {
//         pub(super) texture: Option<gdk::Texture>,
//     }

//     #[glib::object_subclass]
//     impl ObjectSubclass for WgpuPaintable {
//         const NAME: &'static str = "WgpuPaintable";
//         type Type = super::WgpuPaintable;
//         type Interfaces = (gdk::Paintable,);
//         type ParentType = gtk4::Widget;

//         fn class_init(klass: &mut Self::Class) {
//             klass.bind_template();
//         }

//         fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
//             obj.init_template();
//         }
//     }

//     impl WidgetImpl for RnSidebar {}

//     impl ObjectImpl for WgpuPaintable {
//         fn constructed(&self) {
//             self.parent_constructed();
//         }

//         fn dispose(&self) {
//             self.dispose_template();
//             while let Some(child) = self.obj().first_child() {
//                 child.unparent();
//             }
//         }
//     }

//     impl PaintableImpl for WgpuPaintable {
//         fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
//             //
//         }   
//     }

//     impl WgpuPaintable {
//     }

//     impl Default for WgpuPaintable {
//         fn default() -> Self {
//             WgpuPaintable { texture: None }
//         }   
//     }

//     impl WidgetImpl for WgpuPaintable {

//     }
// }

// glib::wrapper! {
//     pub(crate) struct WgpuPaintable(ObjectSubclass<imp::WgpuPaintable>)
//         @implements gdk::Paintable;
// }

// impl Default for WgpuPaintable {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl WgpuPaintable {
//     pub(crate) fn new() -> Self {
//         glib::Object::new()
//     }
// }