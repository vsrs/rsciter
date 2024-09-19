use super::*;

pub struct GlobalAsset<T: HasPassport> {
    ptr: *mut AssetData<T>,
}

impl<T: HasPassport> IAsset for GlobalAsset<T> {
    fn class() -> som_asset_class_t {
        // global assets are not ref-counted.
        unsafe extern "C" fn ref_count_stub(_thing: *mut som_asset_t) -> c_long {
            return 1;
        }

        unsafe extern "C" fn asset_get_interface(
            _thing: *mut som_asset_t,
            _name: *const c_char,
            _out: *mut *mut c_void,
        ) -> c_long {
            // TODO: query interface (any usage?)
            return 0;
        }

        unsafe extern "C" fn asset_get_passport<T: HasPassport>(
            thing: *mut som_asset_t,
        ) -> *mut som_passport_t {
            let asset_ref = AssetRef::<T>::new(thing);
            let Ok(passport) = asset_ref.passport() else {
                return std::ptr::null_mut();
            };
            passport as *const _ as *mut _
        }

        som_asset_class_t {
            asset_add_ref: Some(ref_count_stub),
            asset_release: Some(ref_count_stub),
            asset_get_interface: Some(asset_get_interface),
            asset_get_passport: Some(asset_get_passport::<T>),
        }
    }
    
    fn as_raw_asset(&self) -> &RawAssetObj {
        &(unsafe { &*self.ptr }).obj
    }   
}

impl<T: HasPassport> Drop for GlobalAsset<T> {
    fn drop(&mut self) {
        let ptr: *mut som_asset_t = self.ptr.cast();
        let _res = sapi().and_then(|api| api.release_global_asset(ptr));
        debug_assert!(_res.is_ok());
    }
}

impl<T: HasPassport> GlobalAsset<T> {
    pub fn new(data: T) -> Result<Self> {
        let obj = RawAssetObj::new(Self::class());
        let res = AssetData::new(obj, data);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed);

        // SciterSetGlobalAsset overrides assets, so it might return false only if there is no asset_get_passport callback,
        // as we always provide one, it's safe to ignore the result
        let _res = sapi()?.set_global_asset(ptr as _)?;
        debug_assert!(_res);

        Ok(Self { ptr })
    }

    pub fn as_ref(&self) -> AssetRef<T> {
        unsafe { AssetRef::new(self.ptr.cast()) }
    }
}
