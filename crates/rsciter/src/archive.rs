use crate::api::sapi;
use crate::bindings::{HSARCHIVE, LPCWSTR};
use crate::{Error, Result};

/// Sciter's compressed archive produced by <SKITER_SDK>/bin/<PLATFORM>/packfolder tool
pub struct Archive {
    har: HSARCHIVE,
    #[allow(dead_code)]
    data: Option<Vec<u8>>,
}

impl Archive {
    /// Does not copy the data, works well with [`include_bytes!`] macro
    pub fn open_static(data: &'static [u8]) -> Result<Self> {
        let har = Self::open_archive(data)?;
        Ok(Self { har, data: None })
    }

    /// Saves the data
    pub fn open(data: Vec<u8>) -> Result<Self> {
        let har = Self::open_archive(&data)?;
        Ok(Self {
            har,
            data: Some(data),
        })
    }

    fn open_archive(data: &[u8]) -> Result<HSARCHIVE> {
        let har = sapi()?.open_archive(data)?;

        if har.is_null() {
            Err(Error::InvalidArchive)
        } else {
            Ok(har)
        }
    }

    pub fn get_raw(&self, path: LPCWSTR) -> Result<&[u8]> {
        sapi()?.get_archive_item(self.har, path)
    }

    pub fn get(&self, path: impl AsRef<str>) -> Result<&[u8]> {
        sapi()?.get_archive_item_str(self.har, path)
    }

    pub fn close(&mut self) -> Result<bool> {
        let res = sapi()?.close_archive(self.har)?;
        self.har = std::ptr::null_mut();
        Ok(res)
    }
}

impl Drop for Archive {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
