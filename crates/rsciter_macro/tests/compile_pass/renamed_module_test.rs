use rsciter::{XFunctionProvider, xmod};

#[xmod(RenamedModule)]
mod m {
    pub fn test() {
    }
}

fn main() {
    let mut xfn_provider = RenamedModule;
    let _err = xfn_provider.call("-", &[]).unwrap_err();
}
