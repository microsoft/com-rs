use com::{interfaces, interfaces::IUnknown, Interface};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winapi::shared::{
    basetsd::UINT32,
    dxgi::{CreateDXGIFactory1, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL},
    dxgi1_2::{DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FULLSCREEN_DESC},
    dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
    dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
    minwindef::{FLOAT, UINT},
    windef::HWND,
    winerror::{DXGI_ERROR_UNSUPPORTED, DXGI_STATUS_OCCLUDED, S_OK},
};
use winapi::um::{
    d2d1::{self, *},
    d2d1_1::*,
    d3d11::{D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION},
    d3dcommon::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP},
    dcommon::*,
    errhandlingapi::GetLastError,
    minwinbase::SYSTEMTIME,
    profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency},
    sysinfoapi::GetLocalTime,
    winnt::{self, GUID_SESSION_DISPLAY_STATUS, HRESULT},
    winuser::{RegisterPowerSettingNotification, DEVICE_NOTIFY_WINDOW_HANDLE},
};

use std::ffi::c_void;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// this should most likely just be a wrapper type
// much like we have in winrt
macro_rules! HR {
    ($hr:expr) => {{
        let hr = $hr;
        if hr != 0 {
            panic!("non successful HRESULT 0x{:x}", hr);
        }
    }};
}

macro_rules! check_bool {
    ($bool:expr) => {
        if !$bool.to_bool() {
            #[allow(unused_unsafe)]
            let error = unsafe { GetLastError() };
            panic!(
                "non successful action: {} - 0x{:x}",
                stringify!($bool),
                error
            );
        }
    };
}
pub fn run() {
    com::runtime::init_apartment(com::runtime::ApartmentType::SingleThreaded).unwrap();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("Clock");
    let raw = match window.raw_window_handle() {
        RawWindowHandle::Windows(w) => w,
        _ => panic!("This app only works on Windows"),
    };

    // Create factories
    let d2d_factory = create_d2d_factory();
    let _dxgi_factory = create_dxgi_factory();

    let dpi = get_dpi(&d2d_factory);
    let device_independent_resources = DeviceIndependentResources::new(&d2d_factory);
    let device_dependent_resources =
        DeviceDependentResources::new(&d2d_factory, raw.hwnd as _, dpi);
    let mut clock = Clock::new(
        dpi,
        device_independent_resources,
        device_dependent_resources,
    );
    unsafe {
        check_bool!(RegisterPowerSettingNotification(
            raw.hwnd as _,
            &GUID_SESSION_DISPLAY_STATUS,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        ))
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => clock.render(),
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                clock.render();
            }
            _ => {}
        }
    })
}

trait BoolLike {
    fn to_bool(self) -> bool;
}
impl<T> BoolLike for *mut T {
    fn to_bool(self) -> bool {
        !self.is_null()
    }
}
impl<T> BoolLike for *const T {
    fn to_bool(self) -> bool {
        !self.is_null()
    }
}
macro_rules! primitive_bool {
    ($($t:ty),*) => {
        $(
            impl BoolLike for $t {
                fn to_bool(self) -> bool {
                    self != 0
                }
            }
        )*
    };
}
primitive_bool!(u16, i32);

#[repr(C)]
struct Clock {
    dpi: f32,
    device_independent_resources: DeviceIndependentResources,
    device_dependent_resources: DeviceDependentResources,
}

impl Clock {
    fn new(
        dpi: f32,
        device_independent_resources: DeviceIndependentResources,
        device_dependent_resources: DeviceDependentResources,
    ) -> Self {
        Self {
            dpi,
            device_independent_resources,
            device_dependent_resources,
        }
    }

    fn render(&mut self) {
        unsafe { self.device_dependent_resources.target.BeginDraw() };
        self.draw();
        let hr = unsafe {
            self.device_dependent_resources
                .target
                .EndDraw(std::ptr::null_mut(), std::ptr::null_mut());
            self.device_dependent_resources.swap_chain.Present(1, 0)
        };

        match hr {
            S_OK => {}
            DXGI_STATUS_OCCLUDED => {
                // HR!(self.dx_factory.register_occlusion_status_window(
                //     self.window(),
                //     winapi::um::winuser::WM_USER,
                //     &self.occlusion
                // ));
                // self.visible = false;
            }
            _ => {
                // release_device();
            }
        };
    }

    fn draw(&mut self) {
        let mut orientation = D2D_MATRIX_3X2_F::default();
        orientation.matrix[0][0] = 1.0;
        orientation.matrix[1][1] = 1.0;
        // self.orientation = orientation;
        let offset = d2d1::D2D1_SIZE_F {
            width: 5.0,
            height: 5.0,
        };
        let time = get_time(self.device_independent_resources.animation_frequency);
        unsafe {
            HR!(self
                .device_independent_resources
                .animation_manager
                .Update(time, std::ptr::null_mut()));

            let target = &self.device_dependent_resources.target;
            target.SetUnitMode(D2D1_UNIT_MODE_PIXELS);

            let color_white = D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            target.Clear(&color_white);
            target.SetUnitMode(D2D1_UNIT_MODE_DIPS);

            let mut previous = None;
            target.GetTarget(&mut previous);
            let clock = &self.device_dependent_resources.clock;
            target.SetTarget(clock);
            target.Clear(std::ptr::null());
            self.draw_clock();
            let target = &self.device_dependent_resources.target;
            target.SetTarget(previous.unwrap());

            let clock = &self.device_dependent_resources.clock;

            let mut transform = d2d1::D2D1_MATRIX_3X2_F::default();
            transform.matrix[0][0] = 1.0;
            transform.matrix[1][1] = 1.0;
            transform.matrix[2][0] = offset.width;
            transform.matrix[2][1] = offset.height;

            target.SetTransform(&transform);

            // target.draw_image(
            //     self.shadow.get(),
            //     D2D1_INTERPOLATION_MODE_LINEAR,
            //     D2D1_COMPOSITE_MODE_SOURCE_OVER,
            // );

            let mut identity = D2D_MATRIX_3X2_F::default();
            identity.matrix[0][0] = 1.0;
            identity.matrix[1][1] = 1.0;
            target.SetTransform(&identity);

            target.DrawImage(
                clock,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                D2D1_INTERPOLATION_MODE::default(),
                D2D1_COMPOSITE_MODE::default(),
            );
        }
    }

    fn draw_clock(&mut self) {
        let target = &self.device_dependent_resources.target;
        unsafe {
            let mut size = std::mem::zeroed();
            target.GetSize(&mut size);
            let radius = 200.0f32.max(size.width.min(size.height) / 2.0 - 50.0);
            let offset = d2d1::D2D1_SIZE_F {
                width: 2.0,
                height: 2.0,
            };
            let mut translation = d2d1::D2D1_MATRIX_3X2_F::default();
            translation.matrix[0][0] = 1.0;
            translation.matrix[1][1] = 1.0;
            translation.matrix[2][0] = size.width / offset.width;
            translation.matrix[2][1] = size.height / offset.height;
            target.SetTransform(&translation);
            target.GetTransform(&mut translation);

            let brush = &self.device_dependent_resources.brush;
            let ellipse = D2D1_ELLIPSE {
                point: d2d1::D2D1_POINT_2F::default(),
                radiusX: 50.0,
                radiusY: 50.0,
            };

            target.DrawEllipse(&ellipse, brush, radius / 20.0, None);

            let mut time = SYSTEMTIME::default();
            GetLocalTime(&mut time);

            let second_angle = ((time.wSecond + time.wMilliseconds) as f64 / 1000.0) * 6.0;
            let minute_angle = time.wMinute as f64 * 6.0 + second_angle / 60.0;
            let _hour_angle = (time.wHour % 12) as f64 * 30.0 + minute_angle / 12.0;

            let mut swing = 0.0;
            HR!(self
                .device_independent_resources
                .animation_variable
                .GetValue(&mut swing));

            if 1.0 > swing {
                // static secondPrevious: f64 = second_angle;
                // static minutePrevious: f64 = minute_angle;
                // static hourPrevious: f64 = hour_angle;

                // if (secondPrevious > secondAngle) secondAngle += 360.0f;
                // if (minutePrevious > minuteAngle) minuteAngle += 360.0f;
                // if (hourPrevious > hourAngle)   hourAngle += 360.0f;

                // secondAngle *= static_cast<float>(swing);
                // minuteAngle *= static_cast<float>(swing);
                // hourAngle *= static_cast<float>(swing);
            }

            let mut rotation = d2d1::D2D1_MATRIX_3X2_F::default();
            D2D1MakeRotateMatrix(
                second_angle as f32,
                d2d1::D2D1_POINT_2F::default(),
                &mut rotation,
            );
            let transform = rotation; //* self.orientation * translation;
            target.SetTransform(&transform);

            let zero = d2d1::D2D1_POINT_2F { x: 0.0, y: 0.0 };
            let end = d2d1::D2D1_POINT_2F {
                x: 0.0,
                y: -(radius * 0.75),
            };
            target.DrawLine(
                zero,
                end,
                &self.device_dependent_resources.brush,
                radius / 25.0,
                &self.device_independent_resources.style,
            );

            // m_target->SetTransform(Matrix3x2F::Rotation(minuteAngle) * m_orientation * translation);

            target.DrawLine(
                zero,
                end,
                &self.device_dependent_resources.brush,
                radius / 15.0,
                &self.device_independent_resources.style,
            );

            // m_target->SetTransform(Matrix3x2F::Rotation(hourAngle) * m_orientation * translation);

            let end = d2d1::D2D1_POINT_2F {
                x: 0.0,
                y: -(radius * 0.5),
            };
            target.DrawLine(
                zero,
                end,
                &self.device_dependent_resources.brush,
                radius / 10.0,
                &self.device_independent_resources.style,
            );
        }
    }
}

fn create_swapchain_bitmap(swap_chain: &IDXGISwapChain1, target: &ID2D1DeviceContext) {
    let mut surface: Option<IDXGISurface> = None;
    unsafe {
        HR!(swap_chain.GetBuffer(
            0,
            &IDXGISurface::IID,
            &mut surface as *mut _ as *mut *mut c_void,
        ));

        let mut props = D2D1_BITMAP_PROPERTIES1::default();
        props.pixelFormat = D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        };
        props.bitmapOptions = D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW;

        let mut bitmap = None;

        HR!(target.CreateBitmapFromDxgiSurface(surface.unwrap(), &props, &mut bitmap));
        target.SetTarget(bitmap.unwrap());
    }
}

fn create_swapchain(device: &ID3D11Device, window: HWND) -> IDXGISwapChain1 {
    let factory = get_dxgi_factory(device);

    let mut props = DXGI_SWAP_CHAIN_DESC1::default();
    props.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    props.SampleDesc.Count = 1;
    props.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    props.BufferCount = 2;
    props.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;

    let mut swap_chain = None;

    unsafe {
        HR!(factory.CreateSwapChainForHwnd(
            device,
            window,
            &props,
            std::ptr::null_mut(),
            None,
            &mut swap_chain
        ))
    };

    swap_chain.unwrap()
}

fn get_dxgi_factory(device: &ID3D11Device) -> IDXGIFactory2 {
    let dxdevice = device.query_interface::<IDXGIDevice>().unwrap();
    let mut adapter = None;
    unsafe {
        HR!(dxdevice.GetAdapter(&mut adapter));
        let mut parent = None;
        HR!(adapter.unwrap().GetParent(
            &IDXGIFactory2::IID,
            &mut parent as *mut _ as *mut *mut c_void
        ));
        parent.unwrap()
    }
}

fn create_render_target(factory: &ID2D1Factory1, device: &mut ID3D11Device) -> ID2D1DeviceContext {
    let dxdevice = device.query_interface::<IDXGIDevice>();

    let mut d2device = None;
    let target = unsafe {
        HR!(factory.CreateDevice(&dxdevice, &mut d2device));
        let mut target = None;

        HR!(d2device
            .unwrap()
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &mut target));
        target
    };

    target.unwrap()
}

fn create_device() -> ID3D11Device {
    fn create_device(typ: D3D_DRIVER_TYPE, device: &mut Option<ID3D11Device>) -> HRESULT {
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

        // #ifdef _DEBUG
        //     flags |= D3D11_CREATE_DEVICE_DEBUG;
        // #endif

        unsafe {
            D3D11CreateDevice(
                std::ptr::null_mut(),
                typ,
                std::ptr::null_mut(),
                flags,
                std::ptr::null_mut(),
                0,
                D3D11_SDK_VERSION,
                device as *mut _ as _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }
    }
    let mut device = None;
    let mut hr = create_device(D3D_DRIVER_TYPE_HARDWARE, &mut device);

    if DXGI_ERROR_UNSUPPORTED == hr {
        hr = create_device(D3D_DRIVER_TYPE_WARP, &mut device);
    }

    HR!(hr);
    device.unwrap()
}

fn create_d2d_factory() -> ID2D1Factory1 {
    let options = &D2D1_FACTORY_OPTIONS::default();
    let mut factory = None;
    unsafe {
        HR!(D2D1CreateFactory(
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            &ID2D1Factory1::IID as *const _ as _,
            options,
            &mut factory as *mut _ as _,
        ));
    }
    factory.unwrap()
}

fn create_dxgi_factory() -> IDXGIFactory2 {
    let mut dxgi_factory = None;
    unsafe {
        HR!(CreateDXGIFactory1(
            &IDXGIFactory2::IID as *const _ as _,
            &mut dxgi_factory as *mut _ as _,
        ));
    };
    dxgi_factory.unwrap()
}

fn get_dpi(factory: &ID2D1Factory1) -> f32 {
    let mut dpix: f32 = 0.0;
    let mut dpiy: f32 = 0.0;
    unsafe {
        factory.GetDesktopDpi(&mut dpix, &mut dpiy);
    }
    dpix
}

struct DeviceIndependentResources {
    animation_frequency: winnt::LARGE_INTEGER,
    animation_manager: IUIAnimationManager,
    style: ID2D1StrokeStyle1,
    animation_variable: IUIAnimationVariable,
}

impl DeviceIndependentResources {
    fn new(factory: &ID2D1Factory1) -> Self {
        let mut style_props = D2D1_STROKE_STYLE_PROPERTIES1::default();
        style_props.startCap = D2D1_CAP_STYLE_ROUND;
        style_props.endCap = D2D1_CAP_STYLE_TRIANGLE;

        let mut style = None;
        unsafe {
            HR!(factory.CreateStrokeStyle(&style_props, std::ptr::null_mut(), 0, &mut style));
        }
        let style = style.unwrap();

        let class_id = com::CLSID {
            data1: 0x4C1FC63A,
            data2: 0x695C,
            data3: 0x47E8,
            data4: [0xA3, 0x39, 0x1A, 0x19, 0x4B, 0xE3, 0xD0, 0xB8],
        };
        let animation_manager =
            com::runtime::create_instance::<IUIAnimationManager>(&class_id).unwrap();

        let mut animation_frequency = winnt::LARGE_INTEGER::default();
        let mut animation_variable = None;

        let class_id = com::CLSID {
            // 1D6322AD-AA85-4EF5-A828-86D71067D145
            data1: 0x1D6322AD,
            data2: 0xAA85,
            data3: 0x4EF5,
            data4: [0xA8, 0x28, 0x86, 0xD7, 0x10, 0x67, 0xD1, 0x45],
        };
        let library: IUIAnimationTransitionLibrary =
            com::runtime::create_instance(&class_id).unwrap();
        let mut transition = None;
        unsafe {
            check_bool!(QueryPerformanceFrequency(&mut animation_frequency));

            HR!(library.CreateAccelerateDecelerateTransition(5.0, 1.0, 0.2, 0.8, &mut transition,));

            HR!(animation_manager.CreateAnimationVariable(0.0, &mut animation_variable));
        }
        let animation_variable = animation_variable.unwrap();

        unsafe {
            HR!(animation_manager.ScheduleTransition(
                &animation_variable,
                transition.unwrap(),
                get_time(animation_frequency)
            ));
        }

        Self {
            animation_frequency,
            animation_manager,
            animation_variable,
            style,
        }
    }
}

struct DeviceDependentResources {
    target: ID2D1DeviceContext,
    swap_chain: IDXGISwapChain1,
    clock: ID2D1Bitmap1,
    brush: ID2D1SolidColorBrush,
}

impl DeviceDependentResources {
    fn new(factory: &ID2D1Factory1, window: HWND, dpi: f32) -> Self {
        let mut device = create_device();
        let target = create_render_target(factory, &mut device);
        let swap_chain = create_swapchain(&device, window);
        create_swapchain_bitmap(&swap_chain, &target);

        unsafe { target.SetDpi(dpi, dpi) };

        let brush = create_device_resources(&target);
        let clock = create_device_size_resources(&target, dpi);
        Self {
            target,
            swap_chain,
            brush,
            clock,
        }
    }
}

fn create_device_resources(target: &ID2D1DeviceContext) -> ID2D1SolidColorBrush {
    let color_orange = D2D1_COLOR_F {
        r: 0.92,
        g: 0.38,
        b: 0.208,
        a: 1.0,
    };

    let mut props = D2D1_BRUSH_PROPERTIES::default();
    props.opacity = 0.8;

    let mut brush = None;
    unsafe {
        HR!(target.CreateSolidColorBrush(&color_orange, &props, &mut brush));
    }
    brush.unwrap()
}

fn create_device_size_resources(target: &ID2D1DeviceContext, dpi: f32) -> ID2D1Bitmap1 {
    let size = unsafe {
        let mut size = std::mem::zeroed();
        target.GetSize(&mut size);
        size
    };
    let size = D2D_SIZE_U {
        width: size.width as u32,
        height: size.height as u32,
    };

    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: dpi,
        dpiY: dpi,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
        colorContext: std::ptr::null_mut(),
    };
    let mut clock = None;
    unsafe {
        HR!(target.CreateBitmap(size, std::ptr::null(), 0, &props, &mut clock));
    }

    // m_shadow = nullptr;

    // struct __declspec(uuid("C67EA361-1863-4e69-89DB-695D3E9A5B6B")) Direct2DShadow;

    // check_hresult(m_target->CreateEffect(__uuidof(Direct2DShadow),
    //     m_shadow.put()));

    // m_shadow->SetInput(0, m_clock.get());
    clock.unwrap()
}

fn get_time(frequency: winnt::LARGE_INTEGER) -> f64 {
    let mut time = winnt::LARGE_INTEGER::default();
    unsafe {
        check_bool!(QueryPerformanceCounter(&mut time));
        *time.QuadPart() as f64 / *frequency.QuadPart() as f64
    }
}

interfaces! {
    #[uuid("06152247-6f50-465a-9245-118bfd3b6007")]
    unsafe interface ID2D1Factory: IUnknown {
        fn ReloadSystemMetrics(&self) -> HRESULT;
        fn GetDesktopDpi(&self, dpi_x: *mut FLOAT, dpi_y: *mut FLOAT);
        // ununsed functions
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn f4(&self);
        fn f5(&self);
        fn f6(&self);
        fn f7(&self);
        fn f8(&self);
        fn f9(&self);
        fn f10(&self);
        fn f11(&self);
    }

    #[uuid("bb12d362-daee-4b9a-aa1d-14ba401cfa1f")]
    unsafe interface ID2D1Factory1: ID2D1Factory {
        fn CreateDevice(
            &self,
            dxgi_device: Option<IDXGIDevice>,
            d2d_device: *mut Option<ID2D1Device>,
        ) -> HRESULT;
        fn CreateStrokeStyle(
            &self,
            stroke_style_properties: *const D2D1_STROKE_STYLE_PROPERTIES1,
            dashes: *const FLOAT,
            dashes_count: UINT32,
            stroke_style: *mut Option<ID2D1StrokeStyle1>,
        ) -> HRESULT;
    }

    #[uuid("50c83a1c-e072-4c48-87b0-3630fa36a6d0")]
    unsafe interface IDXGIFactory2: IDXGIFactory1 {
        fn f0(&self);
        fn CreateSwapChainForHwnd(
            &self,
            p_device: IUnknown,
            hwnd: HWND,
            p_desc: *const DXGI_SWAP_CHAIN_DESC1,
            p_fullscreen_desc: *const DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
            p_restrict_to_output: Option<IDXGIOutput>,
            pp_swapchain: *mut Option<IDXGISwapChain1>,
        ) -> HRESULT;
    }

    #[uuid("770aae78-f26f-4dba-a829-253c83d1b387")]
    unsafe interface IDXGIFactory1: IDXGIFactory {
        fn f0(&self);
        fn f1(&self);
    }

    #[uuid("7b7166ec-21c7-44ae-b21a-c9ae321ae369")]
    unsafe interface IDXGIFactory: IDXGIObject {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn f4(&self);
    }

    #[uuid("e8f7fe7a-191c-466d-ad95-975678bda998")]
    unsafe interface ID2D1DeviceContext: ID2D1RenderTarget {
        fn CreateBitmap(
            &self,
            #[pass_through]
            size: d2d1::D2D1_SIZE_U,
            source_data: *const c_void,
            pitch: u32,
            bitmap_properties: *const D2D1_BITMAP_PROPERTIES1,
            bitmap: *mut Option<ID2D1Bitmap1>,
        ) -> HRESULT;
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn CreateBitmapFromDxgiSurface(
            &self,
            surface: IDXGISurface,
            bitmap_properties: *const D2D1_BITMAP_PROPERTIES1,
            bitmap: *mut Option<ID2D1Bitmap1>,
        ) -> HRESULT;
        fn f4(&self);
        fn f5(&self);
        fn f6(&self);
        fn f7(&self);
        fn f8(&self);
        fn f9(&self);
        fn f10(&self);
        fn f11(&self);
        fn f12(&self);
        fn f13(&self);
        fn f14(&self);
        fn SetTarget(&self, image: ID2D1Image);
        fn GetTarget(&self, image: *mut Option<ID2D1Image>);
        fn f15(&self);
        fn f16(&self);
        fn f17(&self);
        fn f18(&self);
        fn SetUnitMode(&self, unit_mode: D2D1_UNIT_MODE);
        fn f19(&self);
        fn f20(&self);
        fn DrawImage(
            &self,
            image: ID2D1Image,
            target_offset: *const d2d1::D2D1_POINT_2F,
            image_rectangle: *const d2d1::D2D1_RECT_F,
            #[pass_through]
            interpolation_mode: D2D1_INTERPOLATION_MODE,
            #[pass_through]
            composite_mode: D2D1_COMPOSITE_MODE,
        );
    }

    #[uuid("47dd575d-ac05-4cdd-8049-9b02cd16f44c")]
    unsafe interface ID2D1Device: ID2D1Resource {
        fn CreateDeviceContext(
            &self,
            options: D2D1_DEVICE_CONTEXT_OPTIONS,
            device_context: *mut Option<ID2D1DeviceContext>,
        ) -> HRESULT;
    }

    #[uuid("2cd90694-12e2-11dc-9fed-001143a055f9")]
    unsafe interface ID2D1RenderTarget: ID2D1Resource {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn CreateSolidColorBrush(
            &self,
            color: *const D2D1_COLOR_F,
            brush_props: *const D2D1_BRUSH_PROPERTIES,
            brush: *mut Option<ID2D1SolidColorBrush>,
        ) -> HRESULT;
        fn f4(&self);
        fn f5(&self);
        fn f6(&self);
        fn f7(&self);
        fn f8(&self);
        fn f9(&self);
        fn DrawLine(
            &self,
            #[pass_through]
            point0: d2d1::D2D1_POINT_2F,
            #[pass_through]
            point1: d2d1::D2D1_POINT_2F,
            brush: ID2D1Brush,
            stroke_width: f32,
            stroke_type: ID2D1StrokeStyle
        );
        fn f10(&self);
        fn f11(&self);
        fn f12(&self);
        fn f13(&self);
        fn DrawEllipse(
            &self,
            ellipse: *const D2D1_ELLIPSE,
            brush: ID2D1Brush,
            stroke_width: f32,
            stroke_style: Option<ID2D1StrokeStyle>,
        );
        fn f14(&self);
        fn f15(&self);
        fn f16(&self);
        fn f17(&self);
        fn f18(&self);
        fn f19(&self);
        fn f20(&self);
        fn f21(&self);
        fn f22(&self);
        fn SetTransform(&self, transform: *const d2d1::D2D1_MATRIX_3X2_F);
        fn GetTransform(&self, transform: *mut d2d1::D2D1_MATRIX_3X2_F);
        fn f23(&self);
        fn f24(&self);
        fn f25(&self);
        fn f26(&self);
        fn f27(&self);
        fn f28(&self);
        fn f29(&self);
        fn f30(&self);
        fn f31(&self);
        fn f32(&self);
        fn f33(&self);
        fn f34(&self);
        fn f35(&self);
        fn f36(&self);
        fn f37(&self);
        fn Clear(&self, clear_color: *const D2D1_COLOR_F);
        fn BeginDraw(&self);
        fn EndDraw(
            &self,
            tag1: *mut D2D1_TAG,
            tag2: *mut D2D1_TAG,
        );
        fn f38(&self);
        fn SetDpi(&self, dpix: f32, dpiy: f32);
        fn f39(&self);
        fn GetSize(&self, ret: *mut d2d1::D2D1_SIZE_F) ;
        fn f40(&self);
        fn f41(&self);
        fn f42(&self);
    }

    #[uuid("2cd90691-12e2-11dc-9fed-001143a055f9")]
    unsafe interface ID2D1Resource: IUnknown {
        fn f0(&self);
    }

    #[uuid("db6f6ddb-ac77-4e88-8253-819df9bbf140")]
    unsafe interface ID3D11Device: IUnknown {}

    #[uuid("54ec77fa-1377-44e6-8c32-88fd5f44c84c")]
    unsafe interface IDXGIDevice: IDXGIObject {
        fn GetAdapter(&self, adapter: *mut Option<IDXGIAdapter>) -> HRESULT;
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
    }

    #[uuid("aec22fb8-76f3-4639-9be0-28eb43a67a2e")]
    unsafe interface IDXGIObject: IUnknown {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn GetParent(
            &self,
            refid: *const com::IID,
            pparent: *mut *mut c_void,
        ) -> HRESULT;
    }

    #[uuid("790a45f7-0d42-4876-983a-0a55cfe6f4aa")]
    unsafe interface IDXGISwapChain1: IDXGISwapChain {}

    #[uuid("310d36a0-d2e7-4c0a-aa04-6a9d23b8886a")]
    unsafe interface IDXGISwapChain: IDXGIDeviceSubObject {
        fn Present(
            &self,
            sync_interval: UINT,
            flags: UINT,
        ) -> HRESULT;
        fn GetBuffer(
            &self,
            buffer: UINT,
            riid: *const com::IID,
            pp_surface: *mut *mut c_void,
        ) -> HRESULT;
    }

    #[uuid("3d3e0379-f9de-4d58-bb6c-18d62992f1a6")]
    unsafe interface IDXGIDeviceSubObject: IDXGIObject {
        fn f0(&self);
    }

    #[uuid("2411e7e1-12ac-4ccf-bd14-9798e8534dc0")]
    unsafe interface IDXGIAdapter: IDXGIObject {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
    }

    #[uuid("ae02eedb-c735-4690-8d52-5a8dc20213aa")]
    unsafe interface IDXGIOutput: IDXGIObject {}

    #[uuid("cafcb56c-6ac3-4889-bf47-9e23bbd260ec")]
    unsafe interface IDXGISurface: IDXGIDeviceSubObject {}

    #[uuid("a898a84c-3873-4588-b08b-ebbf978df041")]
    unsafe interface ID2D1Bitmap1: ID2D1Bitmap {}

    #[uuid("a2296057-ea42-4099-983b-539fb6505426")]
    unsafe interface ID2D1Bitmap: ID2D1Image {}

    #[uuid("65019f75-8da2-497c-b32c-dfa34e48ede6")]
    unsafe interface ID2D1Image: ID2D1Resource {}

    #[uuid("9169896C-AC8D-4e7d-94E5-67FA4DC2F2E8")]
    unsafe interface IUIAnimationManager: IUnknown {
        fn CreateAnimationVariable(
            &self,
            initial_value: f64,
            out: *mut Option<IUIAnimationVariable>,
        ) -> HRESULT;
        fn ScheduleTransition(
            &self,
            var: IUIAnimationVariable,
            transition: IUIAnimationTransition,
            time_now: UI_ANIMATION_SECONDS,
        ) -> HRESULT;
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn Update(&self, time_now: UI_ANIMATION_SECONDS, _ptr: *mut c_void)
            -> HRESULT;
    }

    #[uuid("10a72a66-e91c-43f4-993f-ddf4b82b0b4a")]
    unsafe interface ID2D1StrokeStyle1: ID2D1StrokeStyle {}

    #[uuid("2cd9069d-12e2-11dc-9fed-001143a055f9")]
    unsafe interface ID2D1StrokeStyle: ID2D1Resource {}

    #[uuid("2cd906a9-12e2-11dc-9fed-001143a055f9")]
    unsafe interface ID2D1SolidColorBrush: ID2D1Brush {}

    #[uuid("2cd906a8-12e2-11dc-9fed-001143a055f9")]
    unsafe interface ID2D1Brush: ID2D1Resource {}

    #[uuid("8CEEB155-2849-4ce5-9448-91FF70E1E4D9")]
    unsafe interface IUIAnimationVariable: IUnknown {
        fn GetValue(&self, value: *mut f64) -> HRESULT;
    }

    #[uuid("CA5A14B1-D24F-48b8-8FE4-C78169BA954E")]
    unsafe interface IUIAnimationTransitionLibrary: IUnknown {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn f4(&self);
        fn f5(&self);
        fn f6(&self);
        pub fn CreateAccelerateDecelerateTransition(
            &self,
            duration: UI_ANIMATION_SECONDS,
            fin: f64,
            accel_ratio: f64,
            decel_ratio: f64,
            transition: *mut Option<IUIAnimationTransition>,
        ) -> HRESULT;
    }

    #[uuid("DC6CE252-F731-41cf-B610-614B6CA049AD")]
    unsafe interface IUIAnimationTransition: IUnknown {}
}

#[allow(non_camel_case_types)]
type UI_ANIMATION_SECONDS = f64;
