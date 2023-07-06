#[rsciter_macro::xmod]
mod with_generic {
    pub fn test<T: AsRef<str>>(_s: T) {
    }
}

fn main() {}