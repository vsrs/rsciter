use crate::{args_from_raw_parts, bindings::*, AsAny, Error, Result, Value, WindowState};

pub type EventGroups = EVENT_GROUPS;

pub trait EventHandler<'s>: AsAny {
    fn attached(&'s mut self, he: HELEMENT) {
        let _ = he;
    }
    fn detached(&'s mut self, he: HELEMENT) {
        let _ = he;
    }

    fn subscription(&'s mut self, he: HELEMENT) -> Option<EventGroups> {
        let _ = he;
        Some(EventGroups::HANDLE_ALL)
    }

    fn on_mouse(&'s mut self, he: HELEMENT, mouse: &MOUSE_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = mouse;
        Ok(false)
    }

    fn on_key(&'s mut self, he: HELEMENT, key: &KEY_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = key;
        Ok(false)
    }

    fn on_focus(&'s mut self, he: HELEMENT, params: &FOCUS_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_draw(&'s mut self, he: HELEMENT, params: &DRAW_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_timer(&'s mut self, he: HELEMENT, params: &TIMER_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_event(&'s mut self, he: HELEMENT, params: &BEHAVIOR_EVENT_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_method_call(&'s mut self, he: HELEMENT, params: &METHOD_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_data(&'s mut self, he: HELEMENT, params: &DATA_ARRIVED_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_scroll(&'s mut self, he: HELEMENT, params: &SCROLL_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_size(&'s mut self, he: HELEMENT) -> Result<bool> {
        let _ = he;
        Ok(false)
    }

    fn on_scripting_method_call(
        &'s mut self,
        he: HELEMENT,
        name: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let _ = he;
        let _ = name;
        let _ = args;

        Err(Error::ScriptingNoMethod(name.to_string()))
    }

    fn on_gesture(&'s mut self, he: HELEMENT, params: &GESTURE_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_exchange(&'s mut self, he: HELEMENT, params: &EXCHANGE_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }

    fn on_attribute_change(&'s mut self, he: HELEMENT, params: &ATTRIBUTE_CHANGE_PARAMS) {
        let _ = he;
        let _ = params;
    }

    fn on_som(&'s mut self, he: HELEMENT, params: &SOM_PARAMS) -> Result<bool> {
        let _ = he;
        let _ = params;
        Ok(false)
    }
}

pub(super) unsafe extern "C" fn element_proc_thunk(
    tag: LPVOID,
    he: HELEMENT,
    evtg: UINT,
    params: LPVOID,
) -> SBOOL {
    let _ = tag;
    let _ = he;
    let _ = evtg;
    let _ = params;

    if !tag.is_null() {
        let state_ptr = tag as *mut WindowState;
        if let Some(event_handler) = (*state_ptr).event_handler() {
            let event_group = EVENT_GROUPS(evtg as i32);
            match event_group {
                EVENT_GROUPS::HANDLE_INITIALIZATION => {
                    let params = &*(params as *const INITIALIZATION_PARAMS);
                    if params.cmd == INITIALIZATION_EVENTS::BEHAVIOR_ATTACH as u32 {
                        event_handler.attached(he);
                    } else {
                        event_handler.detached(he);
                    }

                    return true as _;
                }

                EVENT_GROUPS::HANDLE_MOUSE => {
                    let params = &*(params as *const MOUSE_PARAMS);
                    if let Ok(res) = event_handler.on_mouse(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_KEY => {
                    let params = &*(params as *const KEY_PARAMS);
                    if let Ok(res) = event_handler.on_key(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_FOCUS => {
                    let params = &*(params as *const FOCUS_PARAMS);
                    if let Ok(res) = event_handler.on_focus(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_DRAW => {
                    let params = &*(params as *const DRAW_PARAMS);
                    if let Ok(res) = event_handler.on_draw(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_TIMER => {
                    let params = &*(params as *const TIMER_PARAMS);
                    if let Ok(res) = event_handler.on_timer(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_BEHAVIOR_EVENT => {
                    let params = &*(params as *const BEHAVIOR_EVENT_PARAMS);
                    if let Ok(res) = event_handler.on_event(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_METHOD_CALL => {
                    let params = &*(params as *const METHOD_PARAMS);
                    if let Ok(res) = event_handler.on_method_call(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_DATA_ARRIVED => {
                    let params = &*(params as *const DATA_ARRIVED_PARAMS);
                    if let Ok(res) = event_handler.on_data(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_SCROLL => {
                    let params = &*(params as *const SCROLL_PARAMS);
                    if let Ok(res) = event_handler.on_scroll(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_SIZE => {
                    if let Ok(res) = event_handler.on_size(he) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_SCRIPTING_METHOD_CALL => {
                    let params = &mut *(params as *mut SCRIPTING_METHOD_PARAMS);
                    let name = std::ffi::CStr::from_ptr(params.name).to_string_lossy();
                    let args = args_from_raw_parts(params.argv, params.argc);

                    if let Ok(res) = event_handler.on_scripting_method_call(he, &name, args) {
                        if let Some(ret_val) = res {
                            params.result = ret_val.take();
                        }

                        return true as _;
                    };
                }

                EVENT_GROUPS::HANDLE_GESTURE => {
                    let params = &*(params as *const GESTURE_PARAMS);
                    if let Ok(res) = event_handler.on_gesture(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_EXCHANGE => {
                    let params = &*(params as *const EXCHANGE_PARAMS);
                    if let Ok(res) = event_handler.on_exchange(he, params) {
                        return res as _;
                    };
                }

                EVENT_GROUPS::HANDLE_ATTRIBUTE_CHANGE => {
                    let params = &*(params as *const ATTRIBUTE_CHANGE_PARAMS);
                    event_handler.on_attribute_change(he, params);
                }

                EVENT_GROUPS::SUBSCRIPTIONS_REQUEST => {
                    let params = &mut *(params as *mut UINT);
                    if let Some(res) = event_handler.subscription(he) {
                        *params = res.0 as UINT;
                        return true as _;
                    }
                }

                EVENT_GROUPS::HANDLE_SOM => {
                    let params = &*(params as *const SOM_PARAMS);
                    if let Ok(res) = event_handler.on_som(he, params) {
                        return res as _;
                    };
                }
                
                _ => (),
            }
        }
    }

    false as _
}
