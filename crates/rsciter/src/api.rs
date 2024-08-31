#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::{args_as_raw_slice, bindings::*, utf, Error, EventGroups, Result, Value, ValueError};
use std::{ffi::CString, mem::MaybeUninit};

mod graphics;
mod request;

pub use graphics::*;
pub use request::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "dynamic")] {
        pub fn sapi() -> Result<Api<'static>> {
            use std::sync::atomic::{AtomicPtr, Ordering};

            static API: AtomicPtr<ISciterAPI> = AtomicPtr::new(core::ptr::null_mut());

            let mut ptr = API.load(Ordering::Acquire);
            if ptr.is_null() {
                ptr = load_sciter_api()?;
                API.store(ptr, Ordering::Release);
            }

            Ok(Api::from(unsafe { &*ptr }))
        }

        fn load_sciter_api() -> Result<*mut ISciterAPI> {
            unsafe {
                let lib = if let Ok(bin) = std::env::var("SCITER_BIN_FOLDER") {
                    let full = format!("{}/{SCITER_DLL_NAME}", bin);
                    libloading::Library::new(full)
                } else {
                    libloading::Library::new(SCITER_DLL_NAME)
                };
                let lib = lib.map_err(|err| Error::library(SCITER_DLL_NAME, err))?;
                let func: libloading::Symbol<unsafe extern "system" fn() -> *mut ISciterAPI> = lib
                    .get(b"SciterAPI")
                    .map_err(|err| Error::symbol("SciterAPI", err))?;

                let api = func();
                std::mem::forget(lib); // leave loaded forever

                Ok(api)
            }
        }
    }
    else {
        pub fn sapi() -> Result<Api<'static>> {
            let api: &ISciterAPI = unsafe { &*SciterAPI() };

            Ok(Api::from(api))
        }

        #[link(name = "sciter-static-release", kind="static")]
        extern "C" {
            pub fn SciterAPI() -> *const ISciterAPI;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Api<'api> {
    raw: &'api ISciterAPI,
}

impl<'api> From<&'api ISciterAPI> for Api<'api> {
    fn from(value: &'api ISciterAPI) -> Self {
        Self { raw: value }
    }
}

impl<'api> Api<'api> {
    /// API version
    pub fn version(&self) -> u32 {
        self.raw.version
    }

    pub fn class_name(&self) -> Result<Vec<u16>> {
        call_method!(self, SciterClassName as f, {
            Ok(utf::u16_ptr_to_slice(f()).to_vec())
        })
    }

    pub fn sciter_version(&self, kind: VersionKind) -> Result<u32> {
        call_method!(self, SciterVersion(kind.0))
    }

    /// This function is used in response to SCN_LOAD_DATA request.
    ///
    /// Returns `true` if Sciter accepts the data or `false` if error occured
    /// (for example this function was called outside of `SCN_LOAD_DATA` request)
    ///
    /// ### Warning:
    /// If used, call of this function **MUST** be done **ONLY** while handling
    /// `SCN_LOAD_DATA` request and in the same thread.
    /// For asynchronous resource loading use SciterDataReadyAsync
    pub fn data_ready(&self, hwnd: HWND, uri: LPCWSTR, data: &[u8]) -> Result<bool> {
        call_method!(
            self,
            SciterDataReady(hwnd, uri, data.as_ptr(), data.len() as u32) as bool
        )
    }

    /// Use this function outside of SCN_LOAD_DATA request.
    /// This function is needed when you have your own http client implemented in your application.
    ///
    /// Returns `true` if Sciter accepts the data or `false` if error occured
    pub fn data_ready_async(
        &self,
        hwnd: HWND,
        uri: &str,
        data: &[u8],
        request_id: Option<HREQUEST>,
    ) -> Result<bool> {
        let uri = utf::str_to_utf16(uri);
        let request_id = request_id.unwrap_or(std::ptr::null_mut());
        call_method!(
            self,
            SciterDataReadyAsync(
                hwnd,
                uri.as_ptr(),
                data.as_ptr(),
                data.len() as u32,
                request_id
            ) as bool
        )
    }

    pub fn sciter_proc(
        &self,
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<LRESULT> {
        call_method!(self, SciterProc(hwnd, msg, wparam, lparam))
    }

    pub fn sciter_proc_nd(
        &self,
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<(bool, LRESULT)> {
        let mut handled: SBOOL = 0;

        let res = call_method!(self, SciterProcND(hwnd, msg, wparam, lparam, &mut handled))?;
        Ok((handled != 0, res))
    }

    /// Loads HTML file.
    ///
    /// Returns `true` if the text was parsed and loaded successfully, 'false' otherwise.
    pub fn load_file(&self, hwnd: HWND, path: impl AsRef<str>) -> Result<bool> {
        self.load_file_impl(hwnd, path.as_ref())
    }

    fn load_file_impl(&self, hwnd: HWND, path: &str) -> Result<bool> {
        let path = utf::str_to_utf16(path);
        call_method!(self, SciterLoadFile(hwnd, path.as_ptr()) as bool)
    }

    /// Loads an HTML document from memory.
    /// Returns `true` if the document was parsed and loaded successfully, 'false' otherwise.
    pub fn load_html(&self, hwnd: HWND, html: &[u8], base_url: Option<&str>) -> Result<bool> {
        let vec = base_url.map(utf::str_to_utf16).unwrap_or_default();
        let ptr = if vec.is_empty() {
            std::ptr::null()
        } else {
            vec.as_ptr()
        };

        call_method!(
            self,
            SciterLoadHtml(hwnd, html.as_ptr(), html.len() as u32, ptr) as bool
        )
    }

    pub fn set_callback(
        &self,
        hwnd: HWND,
        callback: LPSciterHostCallback,
        param: LPVOID,
    ) -> Result<()> {
        call_method!(self, SciterSetCallback(hwnd, callback, param))
    }

    pub fn set_master_css(&self, css: &str) -> Result<bool> {
        call_method!(
            self,
            SciterSetMasterCSS(css.as_ptr(), css.len() as u32) as bool
        )
    }

    pub fn append_master_css(&self, css: &str) -> Result<bool> {
        call_method!(
            self,
            SciterAppendMasterCSS(css.as_ptr(), css.len() as u32) as bool
        )
    }

    pub fn set_css(&self, hwnd: HWND, css: &str, base_url: &str, media_type: &str) -> Result<bool> {
        let base_utf16 = utf::str_to_utf16(base_url);
        let media_utf16 = utf::str_to_utf16(media_type);
        call_method!(
            self,
            SciterSetCSS(
                hwnd,
                css.as_ptr(),
                css.len() as u32,
                base_utf16.as_ptr(),
                media_utf16.as_ptr()
            ) as bool
        )
    }

    pub fn set_media_type(&self, hwnd: HWND, media_type: &str) -> Result<bool> {
        let media_utf16 = utf::str_to_utf16(media_type);
        call_method!(self, SciterSetMediaType(hwnd, media_utf16.as_ptr()) as bool)
    }

    pub fn set_media_vars(&self, hwnd: HWND, vars: &Value) -> Result<bool> {
        call_method!(self, SciterSetMediaVars(hwnd, &vars.0) as bool)
    }

    pub fn min_width(&self, hwnd: HWND) -> Result<u32> {
        call_method!(self, SciterGetMinWidth(hwnd))
    }

    pub fn min_height(&self, hwnd: HWND, width: u32) -> Result<u32> {
        call_method!(self, SciterGetMinHeight(hwnd, width))
    }

    pub fn call(&self, hwnd: HWND, name: &str, args: &[Value]) -> Result<Value> {
        let name: Vec<i8> = name
            .as_bytes()
            .iter()
            .cloned()
            .chain(std::iter::once(0))
            .map(|b| b as i8)
            .collect();
        let args = args_as_raw_slice(args);
        let mut script_res = crate::Value::new();
        let args_ptr = if args.is_empty() {
            std::ptr::null()
        } else {
            args.as_ptr()
        };
        let res = call_method!(
            self,
            SciterCall(
                hwnd,
                name.as_ptr(),
                args.len() as u32,
                args_ptr,
                &mut script_res.0
            ) as bool
        )?;

        if res {
            Ok(script_res)
        } else {
            Err(Error::EvalFailed)
        }
    }

    pub fn eval(&self, hwnd: HWND, code: &str) -> Result<Value> {
        let code = utf::str_to_utf16_no_trailing_zero(code);
        let mut script_res = crate::Value::new();
        let res = call_method!(
            self,
            SciterEval(hwnd, code.as_ptr(), code.len() as u32, &mut script_res.0) as bool
        )?;
        if res {
            Ok(script_res)
        } else {
            Err(Error::EvalFailed)
        }
    }

    pub fn update_window(&self, hwnd: HWND) -> Result<()> {
        call_method!(self, SciterUpdateWindow(hwnd))
    }

    pub fn translate_message(&self, msg: &mut MSG) -> Result<bool> {
        call_method!(self, SciterTranslateMessage(msg) as bool)
    }

    pub fn set_option(
        &self,
        hwnd: Option<HWND>,
        option: SCITER_RT_OPTIONS,
        value: UINT_PTR,
    ) -> Result<bool> {
        call_method!(
            self,
            SciterSetOption(hwnd.unwrap_or_default(), option as u32, value) as bool
        )
    }

    pub fn get_ppi(&self, hwnd: HWND) -> Result<(u32, u32)> {
        call_method!(self, SciterGetPPI as f, {
            let mut px = MaybeUninit::<u32>::uninit();
            let mut py = MaybeUninit::<u32>::uninit();

            f(hwnd, px.as_mut_ptr(), py.as_mut_ptr());

            Ok((px.assume_init(), py.assume_init()))
        })
    }

    pub fn get_view_expando(&self, hwnd: HWND) -> Result<bool> {
        let _ = hwnd;
        todo!("SciterGetViewExpando");
    }

    pub fn render_d2d(&self, hwnd: HWND, prt: *mut IUnknown) -> Result<bool> {
        // prt - ID2D1RenderTarget
        let _ = hwnd;
        let _ = prt;
        todo!("SciterRenderD2D");
    }

    pub fn d2d_factory(&self) -> Result<bool> {
        // ID2D1Factory
        todo!("SciterD2DFactory");
    }
    pub fn dw_factory(&self) -> Result<bool> {
        // IDWriteFactory
        todo!("SciterDWFactory");
    }

    pub fn graphics_caps(&self) -> Result<u32> {
        call_method!(self, SciterGraphicsCaps as f, {
            let mut caps = MaybeUninit::<u32>::uninit();
            if f(caps.as_mut_ptr()) != 0 {
                Ok(caps.assume_init())
            } else {
                Err(Error::ApiMethodFailed("SciterGraphicsCaps"))
            }
        })
    }

    pub fn set_home_url(&self, hwnd: HWND, base_url: &str) -> Result<bool> {
        let base_u16 = utf::str_to_utf16(base_url);
        call_method!(self, SciterSetHomeURL(hwnd, base_u16.as_ptr()) as bool)
    }

    pub fn create_nsview(&self, mut frame: RECT) -> Result<HWND> {
        call_method!(self, SciterCreateNSView(&mut frame))
    }

    pub fn create_widget(&self, mut frame: RECT) -> Result<HWND> {
        call_method!(self, SciterCreateWidget(&mut frame))
    }

    pub fn create_window(
        &self,
        flags: UINT,
        mut frame: Option<RECT>,
        parent: Option<HWND>,
        delegate: ::std::option::Option<
            unsafe extern "C" fn(
                hwnd: HWND,
                msg: UINT,
                wparam: WPARAM,
                lparam: LPARAM,
                dparam: LPVOID,
                handled: *mut SBOOL,
            ) -> LRESULT,
        >,
        delegate_param: LPVOID,
    ) -> Result<HWND> {
        let frame_ptr = frame
            .as_mut()
            .map(|r| {
                if (r.right - r.left) > 0 {
                    r
                } else {
                    std::ptr::null_mut()
                }
            })
            .unwrap_or(std::ptr::null_mut());

        call_method!(
            self,
            SciterCreateWindow(
                flags,
                frame_ptr,
                delegate,
                delegate_param,
                parent.unwrap_or_default()
            )
        )
    }

    pub fn setup_debug_output(
        &self,
        hwnd: Option<HWND>,
        param: LPVOID,
        proc: DEBUG_OUTPUT_PROC,
    ) -> Result<()> {
        call_method!(
            self,
            SciterSetupDebugOutput(hwnd.unwrap_or_default(), param, proc)
        )
    }
    // --------------- before this line all methods are in order

    pub fn window_attach_event_handler(
        &self,
        hwnd: HWND,
        pep: LPELEMENT_EVENT_PROC,
        tag: LPVOID,
        subscription: EventGroups,
    ) -> Result<()> {
        let res = call_method!(
            self,
            SciterWindowAttachEventHandler(hwnd, pep, tag, subscription.0 as u32)
        )?;

        match res as u32 {
            SCDOM_OK => Ok(()),
            _ => todo!(),
        }
    }

    // --------------- after this line all methods are in order

    /// Initialize VALUE storage
    /// This call has to be made before passing VALUE* to any other functions
    pub fn value_init(&self, value: &mut VALUE) -> Result<()> {
        call_method!(self, ValueInit(value) as VALUE_RESULT as Result<()>)
    }

    /// Clears the VALUE and deallocates all assosiated structures that are not used anywhere else.
    pub fn value_clear(&self, value: &mut VALUE) -> Result<()> {
        call_method!(self, ValueClear(value) as VALUE_RESULT as Result<()>)
    }

    /// Compares two values, returns true if val1 == val2.
    pub fn value_compare(&self, v1: &VALUE, v2: &VALUE) -> Result<bool> {
        call_method!(self, ValueCompare(v1, v2) as VALUE_RESULT as Result<bool>)
    }

    // Copies src VALUE to dst VALUE. dst VALUE must be in ValueInit state.
    pub fn value_copy(&self, dst: &mut VALUE, src: &VALUE) -> Result<()> {
        call_method!(self, ValueCopy(dst, src) as VALUE_RESULT as Result<()>)
    }

    /// Converts T_OBJECT value types to T_MAP or T_ARRAY.
    /// Use this method if you need to pass values between different threads.
    /// The fanction is applicable for the Sciter
    pub fn value_isolate(&self, value: &mut VALUE) -> Result<()> {
        call_method!(self, ValueIsolate(value) as VALUE_RESULT as Result<()>)
    }

    /// Returns VALUE_TYPE and VALUE_UNIT_TYPE flags of the VALUE
    pub fn value_type(&self, value: &VALUE) -> Result<(VALUE_TYPE, VALUE_UNIT_TYPE)> {
        call_method!(self, ValueType as f, {
            let mut atype = MaybeUninit::<UINT>::uninit();
            let mut units = MaybeUninit::<UINT>::uninit();
            let res = VALUE_RESULT(f(value, atype.as_mut_ptr(), units.as_mut_ptr()) as i32);

            match res {
                VALUE_RESULT::HV_OK | VALUE_RESULT::HV_OK_TRUE => {
                    let atype = VALUE_TYPE(atype.assume_init() as i32);
                    let units = VALUE_UNIT_TYPE(units.assume_init() as i32);
                    Ok((atype, units))
                }

                err => Err(Error::from(ValueError::from(err))),
            }
        })
    }

    /// Returns string data for T_STRING type
    /// For T_FUNCTION returns name of the fuction.
    pub fn value_string_data<'v>(&self, value: &'v VALUE) -> Result<&'v [u16]> {
        call_method!(self, ValueStringData as f, {
            let mut chars = MaybeUninit::<LPCWSTR>::uninit();
            let mut count = MaybeUninit::<UINT>::uninit();
            let res = VALUE_RESULT(f(value, chars.as_mut_ptr(), count.as_mut_ptr()) as i32);
            match res {
                VALUE_RESULT::HV_OK => Ok(std::slice::from_raw_parts(
                    chars.assume_init(),
                    count.assume_init() as usize,
                )),
                err => Err(Error::from(ValueError::from(err))),
            }
        })
    }

    /// Sets VALUE to T_STRING type and copies chars/numChars to
    /// internal refcounted buffer assosiated with the value.
    pub fn value_string_data_set(
        &self,
        value: &mut VALUE,
        data: &[u16],
        units: Option<VALUE_UNIT_TYPE>,
    ) -> Result<()> {
        call_method!(
            self,
            ValueStringDataSet(
                value,
                data.as_ptr(),
                data.len() as u32,
                units.unwrap_or(VALUE_UNIT_TYPE(0)).0 as u32
            ) as VALUE_RESULT as Result<()>
        )
    }

    /// Retrieves integer data of T_INT, T_LENGTH and T_BOOL types
    pub fn value_int_data(&self, value: &VALUE) -> Result<i32> {
        value_ret_val!(self, ValueIntData(value) as i32)
    }

    /// Sets VALUE integer data of T_INT and T_BOOL types
    /// Optionally sets units field too.
    pub fn value_int_data_set(
        &self,
        value: &mut VALUE,
        data: i32,
        atype: VALUE_TYPE,
        units: Option<VALUE_UNIT_TYPE>,
    ) -> Result<()> {
        // let units = units.unwrap_or(VALUE_UNIT_TYPE(0));
        // call_method!(self, ValueIntDataSet(value, data, atype.0 as u32, units.0 as u32) as VALUE_RESULT as Result<()>)
        value_impl_set!(self, value, ValueIntDataSet, data, atype, units)
    }

    /// Retrieve 64bit integer data of T_BIG_INT and T_DATE values.
    pub fn value_int64_data(&self, value: &VALUE) -> Result<i64> {
        value_ret_val!(self, ValueInt64Data(value) as i64)
    }

    /// Sets 64bit integer data of T_BIG_INT and T_DATE values.
    /// Optionally sets units field too.
    pub fn value_int64_data_set(
        &self,
        value: &mut VALUE,
        data: i64,
        atype: VALUE_TYPE,
        units: Option<VALUE_UNIT_TYPE>,
    ) -> Result<()> {
        value_impl_set!(self, value, ValueInt64DataSet, data, atype, units)
    }

    /// Retrieve FLOAT_VALUE (double) data of T_FLOAT and T_LENGTH values.
    pub fn value_float_data(&self, value: &VALUE) -> Result<f64> {
        value_ret_val!(self, ValueFloatData(value) as f64)
    }

    /// Sets FLOAT_VALUE data of T_FLOAT and T_LENGTH values.
    /// Optionally sets units field too.
    pub fn value_float_data_set(
        &self,
        value: &mut VALUE,
        data: f64,
        atype: VALUE_TYPE,
        units: Option<VALUE_UNIT_TYPE>,
    ) -> Result<()> {
        value_impl_set!(self, value, ValueFloatDataSet, data, atype, units)
    }

    /// Retrieve integer data of T_BYTES type
    pub fn value_binary_data<'v>(&self, value: &'v VALUE) -> Result<&'v [u8]> {
        call_method!(self, ValueBinaryData as f, {
            let mut bytes = MaybeUninit::<LPCBYTE>::uninit();
            let mut count = MaybeUninit::<UINT>::uninit();
            let res = VALUE_RESULT(f(value, bytes.as_mut_ptr(), count.as_mut_ptr()) as i32);
            match res {
                VALUE_RESULT::HV_OK => Ok(std::slice::from_raw_parts(
                    bytes.assume_init(),
                    count.assume_init() as usize,
                )),
                err => Err(Error::from(ValueError::from(err))),
            }
        })
    }

    /// Sets VALUE to sequence of bytes of type T_BYTES
    pub fn value_binary_data_set(
        &self,
        value: &mut VALUE,
        data: &[u8],
        atype: VALUE_TYPE,
        units: Option<VALUE_UNIT_TYPE>,
    ) -> Result<()> {
        call_method!(
            self,
            ValueBinaryDataSet(
                value,
                data.as_ptr(),
                data.len() as u32,
                atype.0 as u32,
                units.unwrap_or(VALUE_UNIT_TYPE(0)).0 as u32
            ) as VALUE_RESULT as Result<()>
        )
    }

    /// Retrieve number of sub-elements for:
    /// - T_ARRAY - number of elements in the array;
    /// - T_MAP - number of key/value pairs in the map;
    /// - T_FUNCTION - number of arguments in the function;
    pub fn value_elements_count(&self, value: &VALUE) -> Result<usize> {
        value_ret_val!(self, ValueElementsCount(value) as INT).map(|r| r as usize)
    }

    /// Retrieve value of sub-element at index n for:
    /// - T_ARRAY - nth element of the array;
    /// - T_MAP - value of nth key/value pair in the map;
    /// - T_FUNCTION - value of nth argument of the function;
    pub fn value_nth_element_value(&self, value: &VALUE, n: INT) -> Result<VALUE> {
        value_ret_val!(self, ValueNthElementValue(value, n))
    }

    /// sets value of sub-element at index n for:
    /// - T_ARRAY - nth element of the array;
    /// - T_MAP - value of nth key/value pair in the map;
    /// - T_FUNCTION - value of nth argument of the function;
    /// If the VALUE is not of one of types above then it makes it of type T_ARRAY with
    /// single element - 'val_to_set'.
    pub fn value_nth_element_value_set(&self, dst: &mut VALUE, n: INT, src: &VALUE) -> Result<()> {
        call_method!(
            self,
            ValueNthElementValueSet(dst, n, src) as VALUE_RESULT as Result<()>
        )
    }

    pub fn value_nth_element_key(&self, value: &VALUE, n: INT) -> Result<VALUE> {
        value_ret_val!(self, ValueNthElementKey(value, n))
    }

    pub fn value_enum_elements(
        &self,
        value: &VALUE,
        penum: unsafe extern "C" fn(arg1: LPVOID, arg2: *const VALUE, arg3: *const VALUE) -> SBOOL,
        param: LPVOID,
    ) -> Result<()> {
        call_method!(
            self,
            ValueEnumElements(value, Some(penum), param) as VALUE_RESULT as Result<()>
        )
    }

    pub fn value_set_value_to_key(
        &self,
        dst: &mut VALUE,
        key: &VALUE,
        value: &VALUE,
    ) -> Result<()> {
        call_method!(
            self,
            ValueSetValueToKey(dst, key, value) as VALUE_RESULT as Result<()>
        )
    }

    pub fn value_get_value_of_key(self, value: &VALUE, key: &VALUE) -> Result<VALUE> {
        value_ret_val!(self, ValueGetValueOfKey(value, key))
    }

    pub fn value_to_string(&self, value: &mut VALUE, how: VALUE_STRING_CVT_TYPE) -> Result<()> {
        call_method!(
            self,
            ValueToString(value, how.0 as u32) as VALUE_RESULT as Result<()>
        )
    }

    /// Returns Number of non-parsed characters in case of errors.
    /// Thus if string was parsed in full it returns 0 (success)
    pub fn value_from_string(
        &self,
        value: &mut VALUE,
        str: &[u16],
        how: VALUE_STRING_CVT_TYPE,
    ) -> Result<()> {
        let res = call_method!(
            self,
            ValueFromString(value, str.as_ptr(), str.len() as u32, how.0 as u32)
        )?;
        if res == 0 {
            Ok(())
        } else {
            Err(Error::from(ValueError::FromStringNonParsed(res)))
        }
    }

    pub fn value_invoke(
        &self,
        value: &VALUE,
        this: &mut VALUE,
        args: &[VALUE],
        ret_val: &mut VALUE,
        url: LPCWSTR,
    ) -> Result<()> {
        let args_ptr = if args.is_empty() {
            std::ptr::null()
        } else {
            args.as_ptr()
        };
        call_method!(
            self,
            ValueInvoke(value, this, args.len() as u32, args_ptr, ret_val, url as _) as VALUE_RESULT
                as Result<()>
        )
    }

    pub fn value_native_functor_set(
        &self,
        value: &mut VALUE,
        invoke: Option<
            unsafe extern "C" fn(
                tag: *mut ::std::os::raw::c_void,
                argc: UINT,
                argv: *const VALUE,
                retval: *mut VALUE,
            ),
        >,
        release: Option<unsafe extern "C" fn(tag: *mut ::std::os::raw::c_void)>,
        tag: LPVOID,
    ) -> Result<()> {
        call_method!(
            self,
            ValueNativeFunctorSet(value, invoke, release, tag) as VALUE_RESULT as Result<()>
        )
    }

    pub fn value_is_native_functor(&self, value: &VALUE) -> Result<bool> {
        call_method!(self, ValueIsNativeFunctor(value) as VALUE_RESULT).map(|res| res.0 != 0)
        // ValueIsNativeFunctor returns 1 for the native functor
    }

    pub fn atom(&self, name: &str) -> Result<som_atom_t> {
        let cs = CString::new(name).unwrap();
        call_method!(
            self,
            SciterAtomValue(cs.as_ptr() as _) as som_atom_t
        )
    }
    
    pub fn atom_name(&self, id: som_atom_t) -> Option<String> {
        let mut s = String::new();
        let ret = call_method!(
            self,
            SciterAtomNameCB(id, None, &mut s as *mut _ as LPVOID)
        );
        if ret.is_ok_and(|res| res != 0) {
            Some(s)
        }
        else {
            None
        }
    }

    pub fn set_global_asset(&self, som_asset: *mut som_asset_t) -> Result<SBOOL>{
        call_method!(
            self, 
            SciterSetGlobalAsset(som_asset)
        )
    }

    ///////////////////
    pub fn open_archive(&self, data: &[u8]) -> Result<HSARCHIVE> {
        call_method!(self, SciterOpenArchive(data.as_ptr(), data.len() as u32))
    }

    pub fn get_archive_item<'data>(&self, har: HSARCHIVE, path: LPCWSTR) -> Result<&'data [u8]> {
        let mut data: LPCBYTE = std::ptr::null_mut();
        let mut len: UINT = 0;
        let res = call_method!(self, SciterGetArchiveItem(har, path, &mut data, &mut len))?;
        unsafe {
            match res {
                0 => Err(Error::ArchiveItemNotFound(utf::u16_ptr_to_string(path))),
                _ => Ok(std::slice::from_raw_parts(data, len as usize)),
            }
        }
    }

    pub fn close_archive(&self, har: HSARCHIVE) -> Result<bool> {
        call_method!(self, SciterCloseArchive(har) as bool)
    }

    // --------------- before this line methods are in order until the similar comment

    pub fn post_callback(
        &self,
        hwnd: HWND,
        wparam: UINT_PTR,
        lparam: UINT_PTR,
        timeoutms: UINT,
    ) -> Result<UINT_PTR> {
        call_method!(self, SciterPostCallback(hwnd, wparam, lparam, timeoutms))
    }

    pub fn request(&self) -> Result<RequestApi<'api>> {
        call_method!(self, GetSciterRequestAPI as f, {
            Ok(RequestApi::from(&*f()))
        })
    }

    pub fn graphics(&self) -> Result<GraphicsApi<'api>> {
        call_method!(self, GetSciterGraphicsAPI as f, {
            Ok(GraphicsApi::from(&*f()))
        })
    }

    pub fn exec(&self, app_cmd: SCITER_APP_CMD, p1: UINT_PTR, p2: UINT_PTR) -> Result<INT_PTR> {
        let cmd = app_cmd as i32 as UINT;
        call_method!(self, SciterExec(cmd, p1, p2))
    }

    pub fn window_exec(
        &self,
        hwnd: HWND,
        window_cmd: SCITER_WINDOW_CMD,
        p1: UINT_PTR,
        p2: UINT_PTR,
    ) -> Result<INT_PTR> {
        let cmd = window_cmd as i32 as UINT;
        call_method!(self, SciterWindowExec(hwnd, cmd, p1, p2))
    }
}

// user friendly additions
impl<'api> Api<'api> {
    pub fn class_name_as_string(&self) -> Result<String> {
        call_method!(self, SciterClassName as f, {
            let name = utf::u16_ptr_to_slice(f());
            Ok(String::from_utf16_lossy(name))
        })
    }

    /// This function is used in response to SCN_LOAD_DATA request.
    ///
    /// Returns `true` if Sciter accepts the data or `false` if error occured
    /// (for example this function was called outside of `SCN_LOAD_DATA` request)
    ///
    /// ### Warning:
    /// If used, call of this function **MUST** be done **ONLY** while handling
    /// `SCN_LOAD_DATA` request and in the same thread.
    /// For asynchronous resource loading use SciterDataReadyAsync
    pub fn data_ready_str(&self, hwnd: HWND, uri: impl AsRef<str>, data: &[u8]) -> Result<bool> {
        let uri = utf::str_to_utf16(uri.as_ref());
        self.data_ready(hwnd, uri.as_ptr(), data)
    }

    pub fn get_archive_item_str<'data>(
        &self,
        har: HSARCHIVE,
        path: impl AsRef<str>,
    ) -> Result<&'data [u8]> {
        let path = utf::str_to_utf16(path.as_ref());
        self.get_archive_item(har, path.as_ptr())
    }
}

pub struct VersionKind(u32);
impl VersionKind {
    pub const MAJOR: VersionKind = VersionKind(0);
    pub const MINOR: VersionKind = VersionKind(1); // ?? Not sure this names are correct
    pub const UPDATE: VersionKind = VersionKind(2); // ??
    pub const BUILD: VersionKind = VersionKind(3); // ??
    pub const REVISION: VersionKind = VersionKind(4);
}

macro_rules! call_method {
    ($self:ident, $name:ident as $f:ident, $body: block) => {
        $self.raw.$name.ok_or(crate::Error::ApiMethod(stringify!($name))).and_then(|$f| unsafe { $body })
    };

    ($self:ident, $name:ident($( $arg:expr ),*)) => {
        call_method!($self, $name as method, {
            Ok(method($($arg),*))
        })
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as bool) => {
        call_method!($self, $name($($arg),*)).map(|res| res != 0)
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as som_atom_t) => {
        call_method!($self, $name($($arg),*))
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as Result<T>) => {
        call_method!($self, $name($($arg),*))
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as VALUE_RESULT) => {
        call_method!($self, $name($($arg),*)).map(|res| crate::bindings::VALUE_RESULT(res as i32))
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as VALUE_RESULT as Result<()>) => {
        match call_method!($self, $name($($arg),*)).map(|res| crate::bindings::VALUE_RESULT(res as i32))? {
            crate::bindings::VALUE_RESULT::HV_OK | crate::bindings::VALUE_RESULT::HV_OK_TRUE => Ok(()),
            err => Err(crate::Error::from(crate::ValueError::from(err)))
        }
    };

    ($self:ident, $name:ident($( $arg:expr ),*) as VALUE_RESULT as Result<bool>) => {
        match call_method!($self, $name($($arg),*)).map(|res| crate::bindings::VALUE_RESULT(res as i32))? {
            crate::bindings::VALUE_RESULT::HV_OK => Ok(false),
            crate::bindings::VALUE_RESULT::HV_OK_TRUE => Ok(true),
            err => Err(crate::Error::from(crate::ValueError::from(err)))
        }
    };

}

macro_rules! value_impl_set {
    ($self:ident, $value:ident, $name:ident, $data:ident, $atype: expr, $units: expr) => {
        call_method!(
            $self,
            $name(
                $value,
                $data,
                $atype.0 as u32,
                $units.unwrap_or(VALUE_UNIT_TYPE(0)).0 as u32
            ) as VALUE_RESULT as Result<()>
        )
    };
}

macro_rules! value_ret_val {
    ($self:ident, $name:ident ( $( $arg1:expr ),* ; $( $arg2:expr ),* ) as $type:ty) => {
        call_method!($self, $name as f, {
            let mut ret_val = MaybeUninit::<$type>::zeroed();
            let res = VALUE_RESULT(f($($arg1),*, ret_val.as_mut_ptr(), $($arg2),*) as i32);
            match res {
                VALUE_RESULT::HV_OK | VALUE_RESULT::HV_OK_TRUE => Ok(ret_val.assume_init()),
                err => Err(Error::from(ValueError::from(err))),
            }
        })
    };

    ($self:ident, $name:ident ( $( $arg1:expr ),* ; $( $arg2:expr ),* ) ) => {
        value_ret_val!($self, $name ($($arg1),* ; $($arg2),*) as VALUE)
    };

    ($self:ident, $name:ident ( $( $arg:expr ),* ) ) => {
        value_ret_val!($self, $name ($($arg),* ; ) as VALUE)
    };

    ($self:ident, $name:ident ( $( $arg:expr ),* ) as $type:ty ) => {
        value_ret_val!($self, $name ($($arg),* ; ) as $type)
    };
}

// to make visible in submodules and above definitions
pub(self) use call_method;
pub(self) use value_impl_set;
pub(self) use value_ret_val;
