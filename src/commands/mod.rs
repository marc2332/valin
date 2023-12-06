mod font_size;
mod split;

pub use font_size::*;
pub use split::*;

pub trait EditorCommand {
    fn name(&self) -> &str;

    fn run_with_args(&self, args: &str);
}
