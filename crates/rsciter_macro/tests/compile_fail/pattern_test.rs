#[rsciter_macro::xmod]
mod with_pattern {
    pub fn test( &(x, y): &(i32, i32)) {
    }
}

fn main() {}