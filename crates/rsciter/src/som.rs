use std::{ffi::CStr, sync::atomic::{AtomicI32, Ordering}};
use crate::{api::sapi, bindings::*, Result};

pub trait Passport {
	/// A static reference to the passport that describes an asset.
	fn get_passport(&self) -> &'static som_passport_t;
}

/// A non-owning pointer to a native object.
pub struct IAssetRef<T> {
	asset: *mut som_asset_t,
	ty: std::marker::PhantomData<T>,
}

impl<T> Clone for IAssetRef<T> {
	fn clone(&self) -> Self {
		self.add_ref();
		Self {
			asset: self.asset,
			ty: self.ty,
		}
	}
}

impl<T> Drop for IAssetRef<T> {
	fn drop(&mut self) {
		self.release();
	}
}

impl<T> std::fmt::Debug for IAssetRef<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// the current reference count
		self.add_ref();
		let rc = self.release();
		let name = sapi().unwrap().atom_name(self.get_passport().name);
		write!(f, "Asset({}):{}", name.unwrap_or_default(), rc)
	}
}

/// Construct a reference from a managed asset.
impl<T> From<Box<IAsset<T>>> for IAssetRef<T> {
	fn from(asset: Box<IAsset<T>>) -> Self {
		Self::from_raw(IAsset::into_raw(asset))
	}
}

impl<T> IAssetRef<T> {
	/// Get the vtable of an asset.
	fn isa(&self) -> &'static som_asset_class_t {
		unsafe { (*self.asset).isa.as_ref::<'static>().unwrap() }
	}

	/// Increment the reference count of an asset and returns the new value.
	fn add_ref(&self) -> i32 {
		let asset_add_ref = self.isa().asset_add_ref.unwrap();
		unsafe {
			(asset_add_ref)(self.asset)
		}
	}

	/// Decrement the reference count of an asset and returns the new value.
	fn release(&self) -> i32 {
		let asset_release = self.isa().asset_release.unwrap();
		unsafe { (asset_release)(self.asset) }
	}
}

impl<T> IAssetRef<T> {
	/// Construct from a raw pointer, incrementing the reference count.
	pub fn from_raw(asset: *mut som_asset_t) -> Self {
		eprintln!("IAssetRef<{}>::from({:?})", std::any::type_name::<T>(), asset);
		assert!(!asset.is_null());
		let me = Self {
			asset,
			ty: std::marker::PhantomData,
		};
		me.add_ref();
		me
	}

	/// Return the raw pointer, releasing the reference count.
	pub fn into_raw(asset: IAssetRef<T>) -> *mut som_asset_t {
		// decrement reference count
		asset.release();

		// get the pointer and forget about this wrapper
		let ptr = asset.asset;
		std::mem::forget(asset);

		ptr
	}

	/// Get the underlaying pointer.
	pub fn as_ptr(&self) -> *mut som_asset_t {
		self.asset
	}

	/// Get a reference to the underlaying pointer.
	pub fn as_asset(&self) -> &som_asset_t {
		unsafe { & *self.asset }
	}

	/// Get the passport of the asset.
	pub fn get_passport(&self) -> &som_passport_t {
		// TODO: do we need this?
		let ptr = unsafe { (self.isa().asset_get_passport.unwrap())(self.asset) };
		unsafe { & *ptr }
	}
}


/// An owned pointer to a wrapped native object.
#[repr(C)]
pub struct IAsset<T> {
	// NB: should be the first member here
	// in order to `*mut IAsset as *mut som_asset_t` work
	asset: som_asset_t,
	refc: AtomicI32,
	passport: Option<&'static som_passport_t>,
	data: T,
}

/// Make the object to be accessible as other global objects in TIScript.
pub fn set_global<T>(asset: IAssetRef<T>) -> Result<SBOOL> {
	let ptr = asset.as_ptr();
	// eprintln!("IAsset<{}>: {:?}", std::any::type_name::<T>(), ptr);
	sapi()?.set_global_asset(ptr)
}

/// Make the object to be accessible as other global objects in TIScript.
pub fn into_global<T>(asset: Box<IAsset<T>>) -> Result<SBOOL> {
	let ptr = IAsset::into_raw(asset);
	// eprintln!("IAsset<{}>: {:?}", std::any::type_name::<T>(), ptr);
	sapi()?.set_global_asset(ptr)
}

impl<T> std::ops::Deref for IAsset<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T> std::ops::DerefMut for IAsset<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

impl<T> Drop for IAsset<T> {
	fn drop(&mut self) {
		let rc = self.refc.load(Ordering::SeqCst);
		if rc != 0 {
			eprintln!("asset<{}>::drop with {} references alive", std::any::type_name::<T>(), rc);
		}
		assert_eq!(rc, 0);
		// allocated in `iasset::new()`
		let ptr = self.asset.isa as *const som_asset_class_t;
		let ptr = unsafe { Box::from_raw(ptr as *mut som_asset_class_t) };
		drop(ptr);
	}
}

impl<T> IAsset<T> {
	/// Cast the pointer to a managed asset reference.
	#[allow(clippy::mut_from_ref)]
	pub fn from_raw(thing: &*mut som_asset_t) -> &mut IAsset<T> {
		assert!(!thing.is_null());
		// clippy complains about "mut_from_ref".
		// the ref is here just to add a lifetime for our resulting reference
		// not the best design choice though
		unsafe { &mut *(*thing as *mut IAsset<T>) }
	}

	/// Release the pointer.
	fn into_raw(asset: Box<IAsset<T>>) -> *mut som_asset_t {
		let p = Box::into_raw(asset);
		p as *mut som_asset_t
	}
}

impl<T: Passport> IAsset<T> {
	/// Wrap the object into a managed asset.
	pub fn new(data: T) -> Box<Self> {
		// will be freed in `iasset<T>::drop()`
		let isa = Box::new(Self::class());

		let me = Self {
			asset: som_asset_t { isa: Box::leak(isa) },
			refc: Default::default(),
			passport: None,
			data,
		};
		Box::new(me)
	}

	fn class() -> som_asset_class_t {
		extern "C" fn asset_add_ref<T>(thing: *mut som_asset_t) -> i32 {
			{
				let me = IAsset::<T>::from_raw(&thing);
				let t = me.refc.fetch_add(1, Ordering::SeqCst) + 1;
				// eprintln!("iasset<T>::add_ref() -> {}", t);
				return t;
			}
		}
		extern "C" fn asset_release<T>(thing: *mut som_asset_t) -> i32 {
			let t = {
				let me = IAsset::<T>::from_raw(&thing);
				me.refc.fetch_sub(1, Ordering::SeqCst) - 1
			};
			eprintln!("iasset<T>::release() -> {}", t);
			if t == 0 {
				eprintln!("iasset<T>::drop()");
				let me = unsafe { Box::from_raw(thing as *mut IAsset<T>) };
				drop(me);
			}
			return t;
		}
		extern "C" fn asset_get_interface<T>(_thing: *mut som_asset_t, name: LPCSTR, _out: *mut LPVOID) -> i32 {
			if name.is_null() {
				eprintln!("iasset<T>::get_interface({}) is not implemented.", "");
				return 0;
			}
			let cs = unsafe { CStr::from_ptr(name) };
			eprintln!("iasset<T>::get_interface({}) is not implemented.", cs.to_string_lossy().to_owned());
			return 0;
		}
		extern "C" fn asset_get_passport<T: Passport>(thing: *mut som_asset_t) -> *mut som_passport_t
		{
			// here we cache the returned reference in order not to allocate things again
			let me = IAsset::<T>::from_raw(&thing);
			if me.passport.is_none() {
				// eprintln!("asset_get_passport<{}>: {:?}", std::any::type_name::<T>(), thing);
				me.passport = Some(me.data.get_passport());
			}
			let ps = me.passport.as_ref().unwrap();
			return *ps as *const som_passport_t as *mut som_passport_t;
		}

		som_asset_class_t {
			asset_add_ref: Some(asset_add_ref::<T>),
			asset_release: Some(asset_release::<T>),
			asset_get_interface: Some(asset_get_interface::<T>),
			asset_get_passport: Some(asset_get_passport::<T>),
		}	
	}
}

impl Default for som_method_def_t {
	fn default() -> Self {
		Self {
			reserved: std::ptr::null_mut(),
			name: 0,
			params: 0,
			func: None,
		}
	}
}

/// Empty passport.
impl Default for som_passport_t {
	fn default() -> Self {
		use std::ptr;
		Self {
			flags: 0,
			name: 0,

			prop_getter: None,
			prop_setter: None,

			item_getter: None,
			item_setter: None,
			item_next: None,

			properties: ptr::null(),
			n_properties: 0,

			methods: ptr::null(),
			n_methods: 0,
		}
	}
}