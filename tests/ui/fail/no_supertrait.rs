use com::com_interface;

#[com_interface("cc2d05c7-7d20-4ccb-ad75-1e7fb7c77254")]
pub trait LoneInterface {
    fn do_something(&self);
    fn do_other_thing(&self);
}

fn main() {}
