use com::interfaces;

interfaces! {
    #[uuid("cc2d05c7-7d20-4ccb-ad75-1e7fb7c77254")]
    pub unsafe interface LoneInterface {
        unsafe fn do_something(&self);
        unsafe fn do_other_thing(&self);
    }
}

fn main() {}
