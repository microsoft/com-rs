use com::{com_interface, interfaces::IUnknown, ComInterface, ComPtr};
use winapi::shared::minwindef::FLOAT;
use winapi::um::winnt::HRESULT;

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
            let error = unsafe { winapi::um::errhandlingapi::GetLastError() };
            panic!(
                "non successful action: {} - 0x{:x}",
                stringify!($bool),
                error
            );
        }
    };
}

fn main() {
    com::runtime::init_apartment(com::runtime::ApartmentType::SingleThreaded).unwrap();
    let mut clock = ClockWindow::new();

    let mut wc = winapi::um::winuser::WNDCLASSA::default();

    unsafe {
        wc.hCursor =
            winapi::um::winuser::LoadCursorW(std::ptr::null_mut(), winapi::um::winuser::IDC_ARROW);
        wc.hInstance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
        let name = b"Sample\0".as_ptr();
        wc.lpszClassName = name as *const i8;
        wc.style = winapi::um::winuser::CS_HREDRAW | winapi::um::winuser::CS_VREDRAW;
        wc.lpfnWndProc = Some(DesktopWindow::window_proc);

        check_bool!(winapi::um::winuser::RegisterClassA(&wc as *const _));

        let name = b"Sample\0".as_ptr();
        let lp = &mut clock.window as *mut DesktopWindow as _;
        winapi::um::winuser::CreateWindowExA(
            0,
            wc.lpszClassName,
            name as *const i8,
            winapi::um::winuser::WS_OVERLAPPEDWINDOW | winapi::um::winuser::WS_VISIBLE,
            winapi::um::winuser::CW_USEDEFAULT,
            winapi::um::winuser::CW_USEDEFAULT,
            winapi::um::winuser::CW_USEDEFAULT,
            winapi::um::winuser::CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            wc.hInstance,
            lp,
        );
    }

    clock.run();
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

trait Window {
    fn run(&mut self);
}

struct ClockWindow<W> {
    window: W,
}

impl<W: Window> ClockWindow<W> {
    fn run(&mut self) {
        self.window.run();
    }
}

impl ClockWindow<DesktopWindow> {
    fn new() -> Self {
        Self {
            window: DesktopWindow::new(1.3),
        }
    }
}

#[repr(C)]
struct DesktopWindow {
    dpix: f32,
    window: winapi::shared::windef::HWND,
    visible: bool,
    orientation: winapi::um::d2d1::D2D1_MATRIX_3X2_F,
    frequency: winapi::shared::ntdef::LARGE_INTEGER,
    target: Option<ComPtr<ID2D1DeviceContext>>,
    factory: Option<ComPtr<ID2D1Factory1>>,
    swap_chain: Option<ComPtr<IDXGISwapChain1>>,
    manager: Option<ComPtr<IUIAnimationManager>>,
    clock: Option<ComPtr<ID2D1Bitmap1>>,
    style: Option<ComPtr<ID2D1StrokeStyle1>>,
    brush: Option<ComPtr<ID2D1SolidColorBrush>>,
    variable: Option<ComPtr<IUIAnimationVariable>>,
}

impl Default for DesktopWindow {
    fn default() -> Self {
        DesktopWindow {
            window: std::ptr::null_mut(),
            dpix: Default::default(),
            visible: false,
            orientation: winapi::um::d2d1::D2D1_MATRIX_3X2_F::default(),
            frequency: Default::default(),
            target: None,
            factory: None,
            swap_chain: None,
            manager: None,
            clock: None,
            style: None,
            brush: None,
            variable: None,
        }
    }
}

impl DesktopWindow {
    fn new(dpix: f32) -> Self {
        Self {
            dpix,
            ..Default::default()
        }
    }

    unsafe extern "system" fn window_proc(
        window: winapi::shared::windef::HWND,
        message: u32,
        wparam: winapi::shared::minwindef::WPARAM,
        lparam: winapi::shared::minwindef::LPARAM,
    ) -> winapi::shared::minwindef::LRESULT {
        if winapi::um::winuser::WM_NCCREATE == message {
            let cs = lparam as *mut winapi::um::winuser::CREATESTRUCTA;
            let that = (*cs).lpCreateParams as *mut DesktopWindow;
            (*that).window = window;
            winapi::um::winuser::SetWindowLongPtrA(
                window,
                winapi::um::winuser::GWLP_USERDATA,
                that as isize,
            );
        } else {
            let that =
                winapi::um::winuser::GetWindowLongPtrA(window, winapi::um::winuser::GWLP_USERDATA);
            let that = that as usize as *mut DesktopWindow;
            if !that.is_null() {
                return (*that).message_handler(message, wparam, lparam);
            }
        }

        winapi::um::winuser::DefWindowProcA(window, message, wparam, lparam)
    }

    unsafe fn message_handler(
        &mut self,
        message: u32,
        wparam: winapi::shared::minwindef::WPARAM,
        lparam: winapi::shared::minwindef::LPARAM,
    ) -> winapi::shared::minwindef::LRESULT {
        match message {
            winapi::um::winuser::WM_DESTROY => {
                winapi::um::winuser::PostQuitMessage(0);
                0
            }
            winapi::um::winuser::WM_PAINT => {
                let ps = &mut winapi::um::winuser::PAINTSTRUCT::default();
                check_bool!(winapi::um::winuser::BeginPaint(self.window, ps as *mut _));
                self.render();
                check_bool!(!winapi::um::winuser::EndPaint(self.window, ps as *mut _));
                0
            }
            winapi::um::winuser::WM_SIZE => {
                if self.target.is_some() && winapi::um::winuser::SIZE_MINIMIZED != wparam {
                    // resize_swapchain_bitmap();
                    self.render();
                }

                0
            }
            winapi::um::winuser::WM_DISPLAYCHANGE => {
                self.render();
                0
            }
            winapi::um::winuser::WM_USER => {
                // if (S_OK == m_swapChain->Present(0, DXGI_PRESENT_TEST))
                // {
                //     m_dxfactory->UnregisterOcclusionStatus(m_occlusion);
                //     m_occlusion = 0;
                //     m_visible = true;
                // }

                0
            }
            winapi::um::winuser::WM_POWERBROADCAST => {
                let ps = lparam as *const winapi::um::winuser::POWERBROADCAST_SETTING;
                self.visible = (*ps).Data != [0];

                if self.visible {
                    winapi::um::winuser::PostMessageA(
                        self.window,
                        winapi::um::winuser::WM_NULL,
                        0,
                        0,
                    );
                }

                winapi::shared::minwindef::TRUE as isize
            }
            winapi::um::winuser::WM_ACTIVATE => {
                self.visible = !winapi::shared::minwindef::HIWORD(wparam as u32).to_bool();
                0
            }
            winapi::um::winuser::WM_GETMINMAXINFO => {
                let info = lparam as *mut winapi::um::winuser::MINMAXINFO;
                (*info).ptMinTrackSize.y = 200;
                0
            }
            _ => winapi::um::winuser::DefWindowProcA(self.window, message, wparam, lparam),
        }
    }

    fn render(&mut self) {
        let (target, swap_chain) = match self.target {
            None => {
                let mut device = create_device();
                let target = create_render_target(self.factory.as_ref().unwrap(), &mut device);
                self.target = Some(target.clone());
                let swap_chain = create_swapchain(&device, self.window);
                self.swap_chain = Some(swap_chain.clone());

                create_swapchain_bitmap(&swap_chain, &target);

                unsafe { target.set_dpi(self.dpix, self.dpix) };

                self.create_device_resources();
                self.create_device_size_resources();
                (target, swap_chain)
            }
            Some(ref t) => (t.clone(), self.swap_chain.as_ref().unwrap().clone()),
        };

        unsafe { target.begin_draw() };
        self.draw();
        let hr = unsafe {
            target.end_draw(std::ptr::null_mut(), std::ptr::null_mut());
            swap_chain.present(1, 0)
        };

        match hr {
            winapi::shared::winerror::S_OK => {}
            winapi::shared::winerror::DXGI_STATUS_OCCLUDED => {
                // HR!(self.dx_factory.register_occlusion_status_window(
                //     self.window(),
                //     winapi::um::winuser::WM_USER,
                //     &self.occlusion
                // ));
                self.visible = false;
            }
            _ => {
                //     release_device();
            }
        };
    }

    fn draw(&mut self) {
        let mut orientation = winapi::um::dcommon::D2D_MATRIX_3X2_F::default();
        orientation.matrix[0][0] = 1.0;
        orientation.matrix[1][1] = 1.0;
        self.orientation = orientation;
        let offset = winapi::um::d2d1::D2D1_SIZE_F {
            width: 5.0,
            height: 5.0,
        };
        unsafe {
            let time = self.get_time();
            HR!(self
                .manager
                .as_ref()
                .unwrap()
                .update(time, std::ptr::null_mut()));
            let target = self.target.clone().unwrap();
            target.set_unit_mode(winapi::um::d2d1_1::D2D1_UNIT_MODE_PIXELS);
            let color_white = winapi::um::d2d1::D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            target.clear(&color_white);
            target.set_unit_mode(winapi::um::d2d1_1::D2D1_UNIT_MODE_DIPS);
            let mut previous: Option<ID2D1Image> = None;
            target.get_target(&mut previous);
            let clock = self.clock.clone().unwrap();
            target.set_target(clock.get());
            target.clear(std::ptr::null());
            self.draw_clock();
            target.set_target(previous.unwrap());
            let mut transform = winapi::um::d2d1::D2D1_MATRIX_3X2_F::default();
            transform.matrix[0][0] = 1.0;
            transform.matrix[1][1] = 1.0;
            transform.matrix[2][0] = offset.width;
            transform.matrix[2][1] = offset.height;

            target.set_transform(&transform);

            // target.draw_image(
            //     self.shadow.get(),
            //     D2D1_INTERPOLATION_MODE_LINEAR,
            //     D2D1_COMPOSITE_MODE_SOURCE_OVER,
            // );

            let mut identity = winapi::um::dcommon::D2D_MATRIX_3X2_F::default();
            identity.matrix[0][0] = 1.0;
            identity.matrix[1][1] = 1.0;
            target.set_transform(&identity);

            target.draw_image(
                clock.get(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                winapi::um::d2d1_1::D2D1_INTERPOLATION_MODE::default(),
                winapi::um::d2d1_1::D2D1_COMPOSITE_MODE::default(),
            );
        }
    }

    fn draw_clock(&mut self) {
        let target = self.target.as_ref().unwrap();
        unsafe {
            let mut size = std::mem::zeroed();
            target.get_size(&mut size);
            let radius = 200.0f32.max(size.width.min(size.height) / 2.0 - 50.0);
            let offset = winapi::um::d2d1::D2D1_SIZE_F {
                width: 2.0,
                height: 2.0,
            };
            let mut translation = winapi::um::d2d1::D2D1_MATRIX_3X2_F::default();
            translation.matrix[0][0] = 1.0;
            translation.matrix[1][1] = 1.0;
            translation.matrix[2][0] = size.width / offset.width;
            translation.matrix[2][1] = size.height / offset.height;
            target.set_transform(&translation);
            target.get_transform(&mut translation);

            let brush = self.brush.as_ref().map(|b| b.get()).unwrap();
            let ellipse = winapi::um::d2d1::D2D1_ELLIPSE {
                point: winapi::um::d2d1::D2D1_POINT_2F::default(),
                radiusX: 50.0,
                radiusY: 50.0,
            };

            target.draw_ellipse(
                &ellipse,
                brush,
                radius / 20.0,
                Option::<ID2D1StrokeStyle>::None,
            );

            let mut time = winapi::um::minwinbase::SYSTEMTIME::default();
            winapi::um::sysinfoapi::GetLocalTime(&mut time);

            let second_angle = ((time.wSecond + time.wMilliseconds) as f64 / 1000.0) * 6.0;
            let minute_angle = time.wMinute as f64 * 6.0 + second_angle / 60.0;
            let hour_angle = (time.wHour % 12) as f64 * 30.0 + minute_angle / 12.0;

            let mut swing = 0.0;
            HR!(self.variable.as_ref().unwrap().get_value(&mut swing));

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

            let mut rotation = winapi::um::d2d1::D2D1_MATRIX_3X2_F::default();
            winapi::um::d2d1::D2D1MakeRotateMatrix(
                second_angle as f32,
                winapi::um::d2d1::D2D1_POINT_2F::default(),
                &mut rotation,
            );
            let transform = rotation; //* self.orientation * translation;
            target.set_transform(&transform);

            let zero = winapi::um::d2d1::D2D1_POINT_2F { x: 0.0, y: 0.0 };
            let end = winapi::um::d2d1::D2D1_POINT_2F {
                x: 0.0,
                y: -(radius * 0.75),
            };
            target.draw_line(
                zero,
                end,
                self.brush.as_mut().unwrap().get(),
                radius / 25.0,
                self.style.as_mut().unwrap().get(),
            );

            // m_target->SetTransform(Matrix3x2F::Rotation(minuteAngle) * m_orientation * translation);

            target.draw_line(
                zero,
                end,
                self.brush.as_mut().unwrap().get(),
                radius / 15.0,
                self.style.as_mut().unwrap().get(),
            );

            // m_target->SetTransform(Matrix3x2F::Rotation(hourAngle) * m_orientation * translation);

            let end = winapi::um::d2d1::D2D1_POINT_2F {
                x: 0.0,
                y: -(radius * 0.5),
            };
            target.draw_line(
                zero,
                end,
                self.brush.as_mut().unwrap().get(),
                radius / 10.0,
                self.style.as_mut().unwrap().get(),
            );
        }
    }

    fn get_time(&self) -> f64 {
        let mut time = winapi::shared::ntdef::LARGE_INTEGER::default();
        unsafe {
            check_bool!(winapi::um::profileapi::QueryPerformanceCounter(&mut time));
            *time.QuadPart() as f64 / *self.frequency.QuadPart() as f64
        }
    }

    fn create_device_independent_resources(&mut self) {
        let mut style = winapi::um::d2d1_1::D2D1_STROKE_STYLE_PROPERTIES1::default();
        style.startCap = winapi::um::d2d1::D2D1_CAP_STYLE_ROUND;
        style.endCap = winapi::um::d2d1::D2D1_CAP_STYLE_TRIANGLE;

        unsafe {
            HR!(self.factory.as_ref().unwrap().create_stroke_style(
                &style,
                std::ptr::null_mut(),
                0,
                &mut self.style
            ));
        }

        self.schedule_animation();
    }

    fn schedule_animation(&mut self) {
        let class_id = com::CLSID {
            data1: 0x4C1FC63A,
            data2: 0x695C,
            data3: 0x47E8,
            data4: [0xA3, 0x39, 0x1A, 0x19, 0x4B, 0xE3, 0xD0, 0xB8],
        };
        let manager = com::runtime::create_instance::<IUIAnimationManager>(&class_id).unwrap();
        self.manager = Some(manager.clone());

        let class_id = com::CLSID {
            // 1D6322AD-AA85-4EF5-A828-86D71067D145
            data1: 0x1D6322AD,
            data2: 0xAA85,
            data3: 0x4EF5,
            data4: [0xA8, 0x28, 0x86, 0xD7, 0x10, 0x67, 0xD1, 0x45],
        };
        let library: ComPtr<IUIAnimationTransitionLibrary> =
            com::runtime::create_instance(&class_id).unwrap();
        unsafe {
            check_bool!(winapi::um::profileapi::QueryPerformanceFrequency(
                &mut self.frequency
            ));

            let mut transition: Option<IUIAnimationTransition> = None;

            HR!(library.create_accelerate_decelerate_transition(
                5.0,
                1.0,
                0.2,
                0.8,
                &mut transition,
            ));

            HR!(manager.create_animation_variable(0.0, &mut self.variable));

            let variable = self.variable.as_ref().unwrap();

            HR!(manager.schedule_transition(variable, transition.unwrap(), self.get_time()));
        }
    }

    fn create_device_resources(&mut self) {
        let color_orange = winapi::um::d2d1::D2D1_COLOR_F {
            r: 0.92,
            g: 0.38,
            b: 0.208,
            a: 1.0,
        };

        let mut props = winapi::um::d2d1::D2D1_BRUSH_PROPERTIES::default();
        props.opacity = 0.8;

        unsafe {
            let brush = &mut self.brush;

            HR!(self.target.as_ref().unwrap().create_solid_color_brush(
                &color_orange,
                &props,
                brush
            ));
        }
    }

    fn create_device_size_resources(&mut self) {
        let target = self.target.as_ref().unwrap();
        unsafe {
            let mut size = std::mem::zeroed();
            target.get_size(&mut size);
            let size = winapi::um::dcommon::D2D_SIZE_U {
                width: size.width as u32,
                height: size.height as u32,
            };

            let props = winapi::um::d2d1_1::D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: winapi::um::dcommon::D2D1_PIXEL_FORMAT {
                    format: winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: winapi::um::dcommon::D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: self.dpix,
                dpiY: self.dpix,
                bitmapOptions: winapi::um::d2d1_1::D2D1_BITMAP_OPTIONS_TARGET,
                colorContext: std::ptr::null_mut(),
            };

            self.clock = None;

            HR!(target.create_bitmap(size, std::ptr::null(), 0, &props, &mut self.clock));
        }

        // m_shadow = nullptr;

        // struct __declspec(uuid("C67EA361-1863-4e69-89DB-695D3E9A5B6B")) Direct2DShadow;

        // check_hresult(m_target->CreateEffect(__uuidof(Direct2DShadow),
        //     m_shadow.put()));

        // m_shadow->SetInput(0, m_clock.get());
    }
}

fn create_swapchain_bitmap(
    swap_chain: &ComPtr<IDXGISwapChain1>,
    target: &ComPtr<ID2D1DeviceContext>,
) {
    let mut surface: Option<IDXGISurface> = None;
    unsafe {
        HR!(swap_chain.get_buffer(
            0,
            &IDXGISurface::IID as *const _ as *const winapi::shared::guiddef::GUID,
            &mut surface as *mut _ as *mut *mut std::ffi::c_void,
        ));

        let mut props = winapi::um::d2d1_1::D2D1_BITMAP_PROPERTIES1::default();
        props.pixelFormat = winapi::um::dcommon::D2D1_PIXEL_FORMAT {
            format: winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: winapi::um::dcommon::D2D1_ALPHA_MODE_IGNORE,
        };
        props.bitmapOptions = winapi::um::d2d1_1::D2D1_BITMAP_OPTIONS_TARGET
            | winapi::um::d2d1_1::D2D1_BITMAP_OPTIONS_CANNOT_DRAW;

        let mut bitmap: Option<ID2D1Bitmap1> = None;

        HR!(target.create_bitmap_from_dxgi_surface(surface.unwrap(), &props, &mut bitmap));
        target.set_target(bitmap.unwrap());
    }
}

extern "system" {}

fn create_swapchain(
    device: &ComPtr<ID3D11Device>,
    window: winapi::shared::windef::HWND,
) -> ComPtr<IDXGISwapChain1> {
    let factory = get_dxgi_factory(device);

    let mut props = winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1::default();
    props.Format = winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
    props.SampleDesc.Count = 1;
    props.BufferUsage = winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT;
    props.BufferCount = 2;
    props.SwapEffect = winapi::shared::dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;

    let mut swap_chain: Option<IDXGISwapChain1> = None;

    unsafe {
        HR!(factory.create_swap_chain_for_hwnd(
            device.get(),
            window,
            &props,
            std::ptr::null_mut(),
            Option::<IDXGIOutput>::None,
            &mut swap_chain
        ))
    };

    swap_chain.unwrap().upgrade()
}

fn get_dxgi_factory(device: &ComPtr<ID3D11Device>) -> ComPtr<IDXGIFactory2> {
    let dxdevice = device.get_interface::<IDXGIDevice>().unwrap();
    let mut adapter: Option<IDXGIAdapter> = None;
    unsafe {
        HR!(dxdevice.get_adapter(&mut adapter as *mut _));
        let mut parent: Option<IDXGIFactory2> = None;
        HR!(adapter.unwrap().get_parent(
            &IDXGIFactory2::IID as *const _ as *const winapi::shared::guiddef::GUID,
            &mut parent as *mut _ as *mut *mut std::ffi::c_void
        ));
        parent.unwrap().upgrade()
    }
}

fn create_render_target(
    factory: &ComPtr<ID2D1Factory1>,
    device: &mut ComPtr<ID3D11Device>,
) -> ComPtr<ID2D1DeviceContext> {
    let dxdevice = device.get_interface::<IDXGIDevice>();

    let mut d2device: Option<ID2D1Device> = None;
    let target = unsafe {
        HR!(factory.create_device(dxdevice, &mut d2device as *mut _));
        let mut target: Option<ID2D1DeviceContext> = None;

        HR!(d2device.unwrap().create_device_context(
            winapi::um::d2d1_1::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
            &mut target as *mut _
        ));
        target
    };

    ComPtr::new(target.unwrap())
}

fn create_device() -> ComPtr<ID3D11Device> {
    fn create_device(
        typ: winapi::um::d3dcommon::D3D_DRIVER_TYPE,
        device: &mut Option<ComPtr<ID3D11Device>>,
    ) -> HRESULT {
        let flags = winapi::um::d3d11::D3D11_CREATE_DEVICE_BGRA_SUPPORT;

        // #ifdef _DEBUG
        //     flags |= D3D11_CREATE_DEVICE_DEBUG;
        // #endif

        unsafe {
            winapi::um::d3d11::D3D11CreateDevice(
                std::ptr::null_mut(),
                typ,
                std::ptr::null_mut(),
                flags,
                std::ptr::null_mut(),
                0,
                winapi::um::d3d11::D3D11_SDK_VERSION,
                device as *const _ as *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }
    }
    let mut device = None;
    let mut hr = create_device(winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE, &mut device);

    if winapi::shared::winerror::DXGI_ERROR_UNSUPPORTED == hr {
        hr = create_device(winapi::um::d3dcommon::D3D_DRIVER_TYPE_WARP, &mut device);
    }

    HR!(hr);
    device.unwrap()
}

impl Window for DesktopWindow {
    fn run(&mut self) {
        let factory = create_factory().upgrade();
        self.factory = Some(factory.clone());
        let mut dxgi_factory: Option<IDXGIFactory2> = None;
        let _dxgi_factory = unsafe {
            HR!(winapi::shared::dxgi::CreateDXGIFactory1(
                &IDXGIFactory2::IID as *const _ as _,
                &mut dxgi_factory as *mut _ as _,
            ));
            dxgi_factory.unwrap().upgrade()
        };
        let mut dpiy: f32 = 0.0;
        unsafe {
            factory.get_desktop_dpi(&mut self.dpix, &mut dpiy);
            self.create_device_independent_resources();

            check_bool!(winapi::um::winuser::RegisterPowerSettingNotification(
                self.window as _,
                &winapi::um::winnt::GUID_SESSION_DISPLAY_STATUS,
                winapi::um::winuser::DEVICE_NOTIFY_WINDOW_HANDLE,
            ))
        }
        let mut message = winapi::um::winuser::MSG::default();
        loop {
            if self.visible {
                self.render();

                unsafe {
                    while winapi::um::winuser::PeekMessageA(
                        &mut message,
                        std::ptr::null_mut(),
                        0,
                        0,
                        winapi::um::winuser::PM_REMOVE,
                    )
                    .to_bool()
                    {
                        winapi::um::winuser::DispatchMessageA(&message);
                    }
                }
            } else {
                unsafe {
                    let result =
                        winapi::um::winuser::GetMessageA(&mut message, std::ptr::null_mut(), 0, 0);
                    if result.to_bool() {
                        if result != -1 {
                            winapi::um::winuser::DispatchMessageA(&message);
                        }
                    }
                }
            }

            if winapi::um::winuser::WM_QUIT == message.message {
                break;
            }
        }
    }
}

fn create_factory() -> ID2D1Factory1 {
    let fo = &winapi::um::d2d1::D2D1_FACTORY_OPTIONS::default();
    let mut factory: Option<ID2D1Factory1> = None;
    unsafe {
        HR!(winapi::um::d2d1::D2D1CreateFactory(
            winapi::um::d2d1::D2D1_FACTORY_TYPE_SINGLE_THREADED,
            &ID2D1Factory1::IID as *const _ as _,
            fo as *const _,
            &mut factory as *mut _ as _,
        ));
        factory.unwrap()
    }
}

com_interface! {
    #[uuid("06152247-6f50-465a-9245-118bfd3b6007")]
    pub unsafe interface ID2D1Factory: IUnknown {
        fn reload_system_metrics(&self) -> HRESULT;
        fn get_desktop_dpi(&self, dpi_x: *mut FLOAT, dpi_y: *mut FLOAT);
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
        fn f12(&self);
        fn f13(&self);
    }

    #[uuid("bb12d362-daee-4b9a-aa1d-14ba401cfa1f")]
    pub unsafe interface ID2D1Factory1: ID2D1Factory {
        fn create_device(
            &self,
            dxgi_device: Option<IDXGIDevice>,
            d2d_device: *mut Option<ID2D1Device>,
        ) -> HRESULT;
        fn create_stroke_style(
            &self,
            stroke_style_properties: *const winapi::um::d2d1_1::D2D1_STROKE_STYLE_PROPERTIES1,
            dashes: *const FLOAT,
            dashes_count: winapi::shared::basetsd::UINT32,
            stroke_style: *mut Option<ID2D1StrokeStyle1>,
        ) -> HRESULT;
    }

    #[uuid("50c83a1c-e072-4c48-87b0-3630fa36a6d0")]
    pub unsafe interface IDXGIFactory2: IDXGIFactory1 {
        fn gif0(&self);
        fn create_swap_chain_for_hwnd(
            &self,
            p_device: IUnknown,
            hwnd: winapi::shared::windef::HWND,
            p_desc: *const winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
            p_fullscreen_desc: *const winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
            p_restrict_to_output: Option<IDXGIOutput>,
            pp_swapchain: *mut Option<IDXGISwapChain1>,
        ) -> HRESULT;
    }

    #[uuid("770aae78-f26f-4dba-a829-253c83d1b387")]
    pub unsafe interface IDXGIFactory1: IDXGIFactory {
        fn f10(&self);
        fn f11(&self);
    }

    #[uuid("7b7166ec-21c7-44ae-b21a-c9ae321ae369")]
    pub unsafe interface IDXGIFactory: IDXGIObject {
        fn f0(&self);
        fn f1(&self);
        fn f2(&self);
        fn f3(&self);
        fn f4(&self);
    }

    #[uuid("e8f7fe7a-191c-466d-ad95-975678bda998")]
    pub unsafe interface ID2D1DeviceContext: ID2D1RenderTarget {
        fn create_bitmap(
            &self,
            size: winapi::um::d2d1::D2D1_SIZE_U,
            source_data: *const std::ffi::c_void,
            pitch: u32,
            bitmap_properties: *const winapi::um::d2d1_1::D2D1_BITMAP_PROPERTIES1,
            bitmap: *mut Option<ID2D1Bitmap1>,
        ) -> HRESULT;
        fn createbitmapfromwicbitmap(&self);
        fn createcolorcontext(&self);
        fn createcolorcontextfromfilename(&self);
        fn createcolorcontextfromwiccolorcontext(&self);
        fn create_bitmap_from_dxgi_surface(
            &self,
            surface: IDXGISurface,
            bitmap_properties: *const winapi::um::d2d1_1::D2D1_BITMAP_PROPERTIES1,
            bitmap: *mut Option<ID2D1Bitmap1>,
        ) -> HRESULT;
        fn createeffect(&self);
        fn creategradientstopcollection(&self);
        fn createimagebrush(&self);
        fn createbitmapbrush(&self);
        fn createcommandlist(&self);
        fn isdxgiformatsupported(&self);
        fn isbufferprecisionsupported(&self);
        fn getimagelocalbounds(&self);
        fn getimageworldbounds(&self);
        fn getglyphrunworldbounds(&self);
        fn getdevice(&self);
        fn set_target(&self, image: ID2D1Image);
        fn get_target(&self, image: *mut Option<ID2D1Image>);
        fn setrenderingcontrols(&self);
        fn getrenderingcontrols(&self);
        fn setprimitiveblend(&self);
        fn getprimitiveblend(&self);
        fn set_unit_mode(&self, unit_mode: winapi::um::d2d1_1::D2D1_UNIT_MODE);
        fn getunitmode(&self);
        fn drawglyphrun(&self);
        fn draw_image(
            &self,
            image: ID2D1Image,
            target_offset: *const winapi::um::d2d1::D2D1_POINT_2F,
            image_rectangle: *const winapi::um::d2d1::D2D1_RECT_F,
            interpolation_mode: winapi::um::d2d1_1::D2D1_INTERPOLATION_MODE,
            composite_mode: winapi::um::d2d1_1::D2D1_COMPOSITE_MODE,
        );
    }

    #[uuid("47dd575d-ac05-4cdd-8049-9b02cd16f44c")]
    pub unsafe interface ID2D1Device: ID2D1Resource {
        fn create_device_context(
            &self,
            options: winapi::um::d2d1_1::D2D1_DEVICE_CONTEXT_OPTIONS,
            device_context: *mut Option<ID2D1DeviceContext>,
        ) -> HRESULT;
    }

    #[uuid("2cd90694-12e2-11dc-9fed-001143a055f9")]
    pub unsafe interface ID2D1RenderTarget: ID2D1Resource {
        ...4
        fn create_solid_color_brush(
            &self,
            color: *const winapi::um::d2d1::D2D1_COLOR_F,
            brush_props: *const winapi::um::d2d1::D2D1_BRUSH_PROPERTIES,
            brush: *mut Option<ID2D1SolidColorBrush>,
        ) -> HRESULT;
        ...6
        fn draw_line(
            &self,
            point0: winapi::um::d2d1::D2D1_POINT_2F,
            point1: winapi::um::d2d1::D2D1_POINT_2F,
            brush: ID2D1Brush,
            stroke_width: f32,
            stroke_type: ID2D1StrokeStyle
        );
        ...4
        fn draw_ellipse(
            &self,
            ellipse: *const winapi::um::d2d1::D2D1_ELLIPSE,
            brush: ID2D1Brush,
            stroke_width: f32,
            stroke_style: Option<ID2D1StrokeStyle>,
        );
        ...9
        fn set_transform(&self, transform: *const winapi::um::d2d1::D2D1_MATRIX_3X2_F);
        fn get_transform(&self, transform: *mut winapi::um::d2d1::D2D1_MATRIX_3X2_F);
        ...15
        fn clear(&self, clear_color: *const winapi::um::d2d1::D2D1_COLOR_F);
        fn begin_draw(&self);
        fn end_draw(
            &self,
            tag1: *mut winapi::um::d2d1::D2D1_TAG,
            tag2: *mut winapi::um::d2d1::D2D1_TAG,
        );
        ...1
        fn set_dpi(&self, dpix: f32, dpiy: f32);
        ...1
        fn get_size(&self, ret: *mut winapi::um::d2d1::D2D1_SIZE_F) ;
        ...3
    }

    #[uuid("2cd90691-12e2-11dc-9fed-001143a055f9")]
    pub unsafe interface ID2D1Resource: IUnknown {
        fn r0(&self);
    }

    #[uuid("db6f6ddb-ac77-4e88-8253-819df9bbf140")]
    pub unsafe interface ID3D11Device: IUnknown {}

    #[uuid("54ec77fa-1377-44e6-8c32-88fd5f44c84c")]
    pub unsafe interface IDXGIDevice: IDXGIObject {
        fn get_adapter(&self, adapter: *mut Option<IDXGIAdapter>) -> HRESULT;
        fn d2(&self);
        fn d3(&self);
        fn d4(&self);
    }

    #[uuid("aec22fb8-76f3-4639-9be0-28eb43a67a2e")]
    pub unsafe interface IDXGIObject: IUnknown {
        fn o0(&self);
        fn o1(&self);
        fn o2(&self);
        fn get_parent(
            &self,
            refid: winapi::shared::guiddef::REFIID,
            pparent: *mut *mut std::ffi::c_void,
        ) -> HRESULT;
    }

    #[uuid("790a45f7-0d42-4876-983a-0a55cfe6f4aa")]
    pub unsafe interface IDXGISwapChain1: IDXGISwapChain {}

    #[uuid("310d36a0-d2e7-4c0a-aa04-6a9d23b8886a")]
    pub unsafe interface IDXGISwapChain: IDXGIDeviceSubObject {
        fn present(
            &self,
            sync_interval: winapi::shared::minwindef::UINT,
            flags: winapi::shared::minwindef::UINT,
        ) -> HRESULT;
        fn get_buffer(
            &self,
            buffer: winapi::shared::minwindef::UINT,
            riid: winapi::shared::guiddef::REFIID,
            pp_surface: *mut *mut std::ffi::c_void,
        ) -> HRESULT;
    }

    #[uuid("3d3e0379-f9de-4d58-bb6c-18d62992f1a6")]
    pub unsafe interface IDXGIDeviceSubObject: IDXGIObject {
        fn so0(&self);
    }

    #[uuid("2411e7e1-12ac-4ccf-bd14-9798e8534dc0")]
    pub unsafe interface IDXGIAdapter: IDXGIObject {
        fn a0(&self);
        fn a1(&self);
        fn a2(&self);
    }

    #[uuid("ae02eedb-c735-4690-8d52-5a8dc20213aa")]
    pub unsafe interface IDXGIOutput: IDXGIObject {}

    #[uuid("cafcb56c-6ac3-4889-bf47-9e23bbd260ec")]
    pub unsafe interface IDXGISurface: IDXGIDeviceSubObject {}

    #[uuid("a898a84c-3873-4588-b08b-ebbf978df041")]
    pub unsafe interface ID2D1Bitmap1: ID2D1Bitmap {}

    #[uuid("a2296057-ea42-4099-983b-539fb6505426")]
    pub unsafe interface ID2D1Bitmap: ID2D1Image {}

    #[uuid("65019f75-8da2-497c-b32c-dfa34e48ede6")]
    pub unsafe interface ID2D1Image: ID2D1Resource {}

    #[uuid("9169896C-AC8D-4e7d-94E5-67FA4DC2F2E8")]
    pub unsafe interface IUIAnimationManager: IUnknown {
        fn create_animation_variable(
            &self,
            initial_value: f64,
            out: *mut Option<IUIAnimationVariable>,
        ) -> HRESULT;
        fn schedule_transition(
            &self,
            var: IUIAnimationVariable,
            transition: IUIAnimationTransition,
            time_now: UI_ANIMATION_SECONDS,
        ) -> HRESULT;
        fn a2(&self);
        fn a3(&self);
        fn a4(&self);
        fn update(&self, time_now: UI_ANIMATION_SECONDS, _ptr: *mut std::ffi::c_void)
            -> HRESULT;
    }

    #[uuid("10a72a66-e91c-43f4-993f-ddf4b82b0b4a")]
    pub unsafe interface ID2D1StrokeStyle1: ID2D1StrokeStyle {}

    #[uuid("2cd9069d-12e2-11dc-9fed-001143a055f9")]
    pub unsafe interface ID2D1StrokeStyle: ID2D1Resource {}

    #[uuid("2cd906a9-12e2-11dc-9fed-001143a055f9")]
    pub unsafe interface ID2D1SolidColorBrush: ID2D1Brush {}

    #[uuid("2cd906a8-12e2-11dc-9fed-001143a055f9")]
    pub unsafe interface ID2D1Brush: ID2D1Resource {}

    #[uuid("8CEEB155-2849-4ce5-9448-91FF70E1E4D9")]
    pub unsafe interface IUIAnimationVariable: IUnknown {
        fn get_value(&self, value: *mut f64) -> HRESULT;
    }

    #[uuid("CA5A14B1-D24F-48b8-8FE4-C78169BA954E")]
    pub unsafe interface IUIAnimationTransitionLibrary: IUnknown {
        fn a0(&self);
        fn a1(&self);
        fn a2(&self);
        fn a3(&self);
        fn a4(&self);
        fn a5(&self);
        fn a6(&self);
        fn create_accelerate_decelerate_transition(
            &self,
            duration: UI_ANIMATION_SECONDS,
            fin: f64,
            accel_ratio: f64,
            decel_ratio: f64,
            transition: *mut Option<IUIAnimationTransition>,
        ) -> HRESULT;
    }

    #[uuid("DC6CE252-F731-41cf-B610-614B6CA049AD")]
    pub unsafe interface IUIAnimationTransition: IUnknown {}
}

#[allow(non_camel_case_types)]
type UI_ANIMATION_SECONDS = f64;
