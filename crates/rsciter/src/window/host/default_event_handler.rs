use std::collections::HashMap;

use crate::{EventHandler, Result, Value, XFunction, XFunctionProvider};

pub struct DefaultEventHandler {
    functions: HashMap<String, Box<dyn XFunction>>,
    modules: Vec<Box<dyn XFunctionProvider>>,
    custom_handler: Option<Box<dyn for<'s> EventHandler<'s>>>,
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

    pub fn with_module(provider: impl XFunctionProvider) -> Self {
        let boxed: Box<dyn XFunctionProvider> = Box::new(provider);
        Self::new(Default::default(), vec![boxed])
    }

    pub fn xcall(&mut self, name: &str, args: &[Value]) -> Result<Option<Value>> {
        // free functions take precedence
        if let Some(func) = self.functions.get_mut(name) {
            return func.call(args);
        }

        // all modules share single namespace
        for module in self.modules.iter_mut() {
            match module.call(name, args) {
                Ok(res) => return Ok(res),
                Err(crate::Error::ScriptingNoMethod(_)) => { /*try next module */ }
                Err(err) => {
                    // there was a method with such name, but failed
                    return Err(err);
                }
            }
        }

        Err(crate::Error::ScriptingNoMethod(name.to_string()))
    }

    pub fn set_custom_event_handler(&mut self, handler: Box<dyn for<'s> EventHandler<'s>>) {
        self.custom_handler = Some(handler);
    }
}

impl<'s> EventHandler<'s> for DefaultEventHandler {
    fn attached(&'s mut self, he: crate::bindings::HELEMENT) {
        if let Some(handler) = self.custom_handler.as_mut() {
            handler.attached(he);
        }
    }

    fn detached(&'s mut self, he: crate::bindings::HELEMENT) {
        if let Some(handler) = self.custom_handler.as_mut() {
            handler.detached(he);
        }
    }

    fn subscription(&'s mut self, he: crate::bindings::HELEMENT) -> Option<crate::EventGroups> {
        self.custom_handler
            .as_mut()
            .and_then(move |it| it.subscription(he))
            .or(Some(crate::EventGroups::HANDLE_ALL))
    }

    fn on_mouse(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        mouse: &crate::bindings::MOUSE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_mouse(he, mouse))
            .unwrap_or(Ok(false))
    }

    fn on_key(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        key: &crate::bindings::KEY_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_key(he, key))
            .unwrap_or(Ok(false))
    }

    fn on_focus(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::FOCUS_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_focus(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_draw(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::DRAW_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_draw(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_timer(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::TIMER_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_timer(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_event(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::BEHAVIOR_EVENT_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_event(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_method_call(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::METHOD_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_method_call(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_data(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::DATA_ARRIVED_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_data(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_scroll(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::SCROLL_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_scroll(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_size(&'s mut self, he: crate::bindings::HELEMENT) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_size(he))
            .unwrap_or(Ok(false))
    }

    fn on_scripting_method_call(
        &'s mut self,
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
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::GESTURE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_gesture(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_exchange(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::EXCHANGE_PARAMS,
    ) -> crate::Result<bool> {
        self.custom_handler
            .as_mut()
            .map(move |it| it.on_exchange(he, params))
            .unwrap_or(Ok(false))
    }

    fn on_attribute_change(
        &'s mut self,
        he: crate::bindings::HELEMENT,
        params: &crate::bindings::ATTRIBUTE_CHANGE_PARAMS,
    ) {
        if let Some(custom) = self.custom_handler.as_mut() {
            custom.on_attribute_change(he, params);
        }
    }

    fn on_passport(
        &'s mut self,
        he: crate::bindings::HELEMENT,
    ) -> Result<Option<&'s crate::bindings::som_passport_t>> {
        let _ = he;
        Ok(None)
    }

    fn on_asset(
        &'s mut self,
        he: crate::bindings::HELEMENT,
    ) -> Result<Option<&'s crate::bindings::som_asset_t>> {
        let _ = he;
        Ok(None)
    }
}
