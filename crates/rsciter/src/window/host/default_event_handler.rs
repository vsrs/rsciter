use std::collections::HashMap;

use crate::{EventHandler, Result, Value, XFunction, XFunctionProvider};

pub struct DefaultEventHandler {
    functions: HashMap<String, Box<dyn XFunction>>,
    modules: Vec<Box<dyn XFunctionProvider>>,
    custom_handler: Option<Box<dyn EventHandler>>,
}

impl DefaultEventHandler {
    pub fn new(
        functions: HashMap<String, Box<dyn XFunction>>,
        modules: Vec<Box<dyn XFunctionProvider>>,
    ) -> Self {
        Self {
            functions,
            modules,
            custom_handler: None,
        }
    }

    pub fn xcall(&mut self, name: &str, args: &[Value]) -> Result<Option<Value>> {
        // free functions take precedence
        if let Some(func) = self.functions.get_mut(name) {
            return func.call(args);
        }

        // all modules share single namespace
        for module in self.modules.iter_mut() {
            let res = module.call(name, args);
            if res.is_ok() {
                return res;
            }
        }

        Err(crate::Error::ScriptingNoMethod(name.to_string()))
    }

    pub fn set_custom_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.custom_handler = Some(handler);
    }
}

impl EventHandler for DefaultEventHandler {
    fn attached(&mut self, he: crate::bindings::HELEMENT) {
        if let Some(handler) = self.custom_handler.as_mut() {
            handler.attached(he);
        }
    }

    fn detached(&mut self, he: crate::bindings::HELEMENT) {
        if let Some(handler) = self.custom_handler.as_mut() {
            handler.detached(he);
        }
    }

    fn subscription(&mut self, he: crate::bindings::HELEMENT) -> Option<crate::EventGroups> {
        self.custom_handler
            .as_mut()
            .and_then(move |it| it.subscription(he))
            .or(Some(crate::EventGroups::HANDLE_ALL))
    }

    fn on_mouse(
        &mut self,
        he: crate::bindings::HELEMENT,
        mouse: &crate::bindings::MOUSE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_mouse(he, mouse))
            .unwrap_or(Ok(false))
    }

    fn on_key(
        &mut self,
        he: crate::bindings::HELEMENT,
        key: &crate::bindings::KEY_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_key(he, key))
            .unwrap_or(Ok(false))
    }

    fn on_focus(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::FOCUS_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_focus(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_draw(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::DRAW_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_draw(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_timer(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::TIMER_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_timer(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_event(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::BEHAVIOR_EVENT_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_event(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_method_call(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::METHOD_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_method_call(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_data(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::DATA_ARRIVED_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_data(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_scroll(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::SCROLL_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_scroll(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_size(&mut self, he: crate::bindings::HELEMENT) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_size(he))
            .unwrap_or(Ok(false))
    }

    fn on_scripting_method_call(
        &mut self,
        he: crate::bindings::HELEMENT,
        name: &str,
        args: &[crate::Value],
    ) -> crate::Result<Option<crate::Value>> {
        if let Some(custom) = self.custom_handler.as_mut() {
            let res = custom.on_scripting_method_call(he, name, args);
            if res.is_ok() {
                return res;
            }
        }

        self.xcall(name, args)
    }

    fn on_gesture(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::GESTURE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_gesture(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_exchange(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::EXCHANGE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_exchange(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_attribute_change(
        &mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::ATTRIBUTE_CHANGE_PARAMS,
    ) {
        if let Some(custom) = self.custom_handler.as_mut() {
            custom.on_attribute_change(he, params);
        }
    }
}
