use std::iter;
use console::Term;

pub fn title() {
    println!("Save &\n   Execute v{}   (c) Brandon Fowler", env!("CARGO_PKG_VERSION"));
}

pub fn create() {
    let height = Term::stdout().size().0 as usize;

    let _ = Term::stdout().write_str(&(iter::repeat(&'\n').take(height - 1).collect::<String>()));
    let _ = Term::stdout().move_cursor_to(0, 0);
}

pub fn clear() {
    let _ = Term::stdout().move_cursor_to(0, 0);
    let _ = Term::stdout().clear_to_end_of_screen();
}