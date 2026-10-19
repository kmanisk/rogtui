use std::ffi::{c_int, c_uint, c_void};
use std::sync::Mutex;
use std::time::{Duration, Instant};

type NvmlReturn = c_int;
type NvmlDevice = *mut c_void;

const NVML_SUCCESS: NvmlReturn = 0;
const NVML_TEMPERATURE_GPU: c_uint = 0;

#[repr(C)]
struct NvmlUtilization {
    gpu: c_uint,
    memory: c_uint,
}

struct NvmlFunctions {
    handle: *mut c_void,
    _init: unsafe extern "C" fn() -> NvmlReturn,
    shutdown: unsafe extern "C" fn() -> NvmlReturn,
    get_handle_by_index: unsafe extern "C" fn(c_uint, *mut NvmlDevice) -> NvmlReturn,
    get_temp: unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint) -> NvmlReturn,
    get_utilization: unsafe extern "C" fn(NvmlDevice, *mut NvmlUtilization) -> NvmlReturn,
    get_power_usage: Option<unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn>,
}

unsafe impl Send for NvmlFunctions {}
unsafe impl Sync for NvmlFunctions {}

impl Drop for NvmlFunctions {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.shutdown)();
            libc::dlclose(self.handle);
        }
    }
}

type GpuMetricsCache = (Instant, Option<u32>, Option<u32>, Option<f32>);

pub struct NvmlReader {
    funcs: Option<NvmlFunctions>,
    last_query: Mutex<Option<GpuMetricsCache>>,
}

impl Default for NvmlReader {
    fn default() -> Self {
        Self::new()
    }
}

impl NvmlReader {
    pub fn new() -> Self {
        let funcs = Self::load_library();
        Self {
            funcs,
            last_query: Mutex::new(None),
        }
    }

    fn load_library() -> Option<NvmlFunctions> {
        let lib_names = ["libnvidia-ml.so.1\0", "libnvidia-ml.so\0"];
        let mut handle = std::ptr::null_mut();

        for name in lib_names {
            handle = unsafe { libc::dlopen(name.as_ptr() as *const libc::c_char, libc::RTLD_NOW) };
            if !handle.is_null() {
                break;
            }
        }

        if handle.is_null() {
            return None;
        }

        unsafe {
            macro_rules! load_sym {
                ($sym:expr, $ty:ty) => {{
                    let s = libc::dlsym(handle, $sym.as_ptr() as *const libc::c_char);
                    if s.is_null() {
                        libc::dlclose(handle);
                        return None;
                    }
                    std::mem::transmute::<*mut c_void, $ty>(s)
                }};
            }

            let init = load_sym!(b"nvmlInit_v2\0", unsafe extern "C" fn() -> NvmlReturn);
            let shutdown = load_sym!(b"nvmlShutdown\0", unsafe extern "C" fn() -> NvmlReturn);
            let get_handle_by_index = load_sym!(
                b"nvmlDeviceGetHandleByIndex_v2\0",
                unsafe extern "C" fn(c_uint, *mut NvmlDevice) -> NvmlReturn
            );
            let get_temp = load_sym!(
                b"nvmlDeviceGetTemperature\0",
                unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint) -> NvmlReturn
            );
            let get_utilization = load_sym!(
                b"nvmlDeviceGetUtilizationRates\0",
                unsafe extern "C" fn(NvmlDevice, *mut NvmlUtilization) -> NvmlReturn
            );
            let get_power_usage = {
                let s = libc::dlsym(handle, c"nvmlDeviceGetPowerUsage".as_ptr());
                if s.is_null() {
                    None
                } else {
                    Some(std::mem::transmute::<*mut c_void, unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn>(s))
                }
            };

            let ret = init();
            if ret != NVML_SUCCESS {
                libc::dlclose(handle);
                return None;
            }

            Some(NvmlFunctions {
                handle,
                _init: init,
                shutdown,
                get_handle_by_index,
                get_temp,
                get_utilization,
                get_power_usage,
            })
        }
    }

    pub fn query_gpu_metrics(&self) -> (Option<u32>, Option<u32>, Option<f32>) {
        let funcs = match self.funcs.as_ref() {
            Some(f) => f,
            None => return (None, None, None),
        };

        // Cache for 3 seconds to keep polling at 2-5s intervals without spawning processes
        let mut cache = self.last_query.lock().unwrap();
        if let Some((ts, temp, util, power)) = *cache {
            if ts.elapsed() < Duration::from_secs(3) {
                return (temp, util, power);
            }
        }

        unsafe {
            let mut device: NvmlDevice = std::ptr::null_mut();
            if (funcs.get_handle_by_index)(0, &mut device) != NVML_SUCCESS {
                *cache = Some((Instant::now(), None, None, None));
                return (None, None, None);
            }

            let mut temp_val: c_uint = 0;
            let temp_opt = if (funcs.get_temp)(device, NVML_TEMPERATURE_GPU, &mut temp_val) == NVML_SUCCESS {
                Some(temp_val as u32)
            } else {
                None
            };

            let mut util_raw = NvmlUtilization { gpu: 0, memory: 0 };
            let util_opt = if (funcs.get_utilization)(device, &mut util_raw) == NVML_SUCCESS {
                Some(util_raw.gpu)
            } else {
                None
            };

            let power_opt = if let Some(get_pwr) = funcs.get_power_usage {
                let mut pwr_val: c_uint = 0;
                if (get_pwr)(device, &mut pwr_val) == NVML_SUCCESS {
                    Some(pwr_val as f32 / 1000.0)
                } else {
                    None
                }
            } else {
                None
            };

            *cache = Some((Instant::now(), temp_opt, util_opt, power_opt));
            (temp_opt, util_opt, power_opt)
        }
    }
}
