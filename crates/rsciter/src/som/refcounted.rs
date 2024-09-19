use super::*;

pub struct Asset<T: HasPassport> {
    boxed: Box<AssetDataWithCounter<T>>,
}

#[repr(C)]
struct AssetDataWithCounter<T> {
    data: AssetData<T>,
    counter: std::sync::atomic::AtomicI32,
}

impl<T> AssetDataWithCounter<T> {
    unsafe fn get_mut_ref<'c>(thing: *mut som_asset_t) -> &'c mut Self {
        let this = thing as *mut AssetDataWithCounter<T>;
        &mut *this
    }
}

impl<T: HasPassport> IAsset for Asset<T> {
    fn class() -> som_asset_class_t {
        unsafe extern "C" fn asset_add_ref<TT>(thing: *mut som_asset_t) -> c_long {
            let this = AssetDataWithCounter::<TT>::get_mut_ref(thing);
            let refc = this.counter.fetch_add(1, Ordering::SeqCst) + 1;

            println!("+{thing:?} {refc}");

            return refc;
        }

        unsafe extern "C" fn asset_release<TT>(thing: *mut som_asset_t) -> c_long {
            println!("-{thing:?}");

            let this = AssetDataWithCounter::<TT>::get_mut_ref(thing);
            let refc = this.counter.fetch_sub(1, Ordering::SeqCst) - 1;
            println!("-{thing:?} {refc}");
            if refc == 0 {
                // TODO: should not panic, reason: for each asset we got unexpected asset_release call with bad ptr
                let _ = std::panic::catch_unwind(|| {
                    let _asset_to_drop = Box::from_raw(thing as *mut AssetDataWithCounter<TT>);
                });
            }
            return refc;
        }

        unsafe extern "C" fn asset_get_interface(
            _thing: *mut som_asset_t,
            _name: *const c_char,
            _out: *mut *mut c_void,
        ) -> c_long {
            // TODO: query interface (any usage?)
            return 0;
        }

        unsafe extern "C" fn asset_get_passport<TT: HasPassport>(
            thing: *mut som_asset_t,
        ) -> *mut som_passport_t {
            let asset_ref = AssetRef::<TT>::new(thing);
            let Ok(passport) = asset_ref.passport() else {
                return std::ptr::null_mut();
            };
            passport as *const _ as *mut _
        }

        som_asset_class_t {
            asset_add_ref: Some(asset_add_ref::<T>),
            asset_release: Some(asset_release::<T>),
            asset_get_interface: Some(asset_get_interface),
            asset_get_passport: Some(asset_get_passport::<T>),
        }
    }
    
    fn as_raw_asset(&self) -> &RawAssetObj {
        &self.boxed.data.obj
    }    
}

impl<T: HasPassport> Asset<T> {
    pub fn new(data: T) -> Self {
        let obj = RawAssetObj::new(Self::class());
        Self {
            boxed: Box::new(AssetDataWithCounter {
                data: AssetData::new(obj, data),
                counter: Default::default(),
            }),
        }
    }

    pub(crate) fn to_raw_ptr(self) -> *const som_asset_t {
        let ptr = Box::into_raw(self.boxed);
        ptr.cast()
    }
}
