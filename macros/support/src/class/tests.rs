use crate::test_utils::is_verbose_testing;
use crate::test_utils::rustfmt;
use crate::Class;
use proc_macro2::TokenStream;
use quote::quote;

fn parse_class_ok(input: TokenStream) -> Class {
    match syn::parse2::<Class>(input.clone()) {
        Ok(class) => {
            // For any class that successfully parses, we expect that we can also
            // generate valid output tokens.
            let output_tokens = class.to_tokens();
            if is_verbose_testing() {
                let formatted_output = rustfmt::run(&output_tokens.to_string());
                println!("output:\n{}", formatted_output);
            }
            class
        }
        Err(e) => {
            panic!(
                "Expected class definition to parse successfully.\nClass: {}\nError: {:?}",
                input, e
            );
        }
    }
}

fn parse_class_err(input: TokenStream, expected_error: &str) {
    assert!(!expected_error.is_empty());

    match syn::parse2::<Class>(input.clone()) {
        Ok(_) => {
            panic!(
                "Expected class definition to fail to parse.\nInput: {}",
                input
            );
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string.contains(expected_error) {
                panic!(
                    "Did not find expected error string.\nActual error: {:?}\nExpected error: {:?}",
                    e_string, expected_error
                );
            }
        }
    }
}

#[test]
fn docs() {
    let class = parse_class_ok(quote! {
        /// Something interesting.
        #[doc = "And another thing!"]
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert_eq!(class.docs.len(), 2);
    for attr in class.docs.iter() {
        assert!(attr.path.is_ident("doc"));
    }
    assert_eq!(
        class.docs[0].tokens.to_string(),
        "= r\" Something interesting.\""
    );
    assert_eq!(class.docs[1].tokens.to_string(), "= \"And another thing!\"");
}

#[test]
fn no_factory() {
    let class = parse_class_ok(quote! {
        #[no_class_factory]
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert!(!class.has_class_factory);
}

#[test]
fn has_factory() {
    let class = parse_class_ok(quote! {
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert!(class.has_class_factory);
}

#[test]
fn class_name() {
    let class = parse_class_ok(quote! {
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert!(class.name == "Simple");
}

#[test]
fn interface_list() {
    let class = parse_class_ok(quote! {
        pub class Simple: IFoo(IZap), IBar {}
        impl IFoo for Simple {}
        impl IBar for Simple {}
        impl IZap for Simple {}
    });
    assert_eq!(class.interfaces.len(), 2);
    let interface0 = &class.interfaces[0];
    assert!(interface0.path.is_ident("IFoo"));
    assert!(interface0.parent.is_some());
    assert!(interface0.parent.as_ref().unwrap().path.is_ident("IZap"));
    let interface1 = &class.interfaces[1];
    assert!(interface1.path.is_ident("IBar"));
    assert!(interface1.parent.is_none());
}

#[test]
fn parse_class_err_no_interfaces() {
    parse_class_err(
        quote! {
            pub class Simple {}
        },
        "expected `:`",
    );
}

#[test]
fn err_no_impl() {
    parse_class_err(
        quote! {
            pub class Simple: IFoo {}
        },
        "impl for interface is missing",
    );
}

#[test]
fn err_missing_indirect_impl() {
    parse_class_err(
        quote! {
            pub class Simple: IFoo(IBar) {}
            impl IFoo for Simple {}
        },
        "impl for interface is missing",
    );
}

#[test]
fn err_impl() {
    parse_class_err(
        quote! {
            pub class Simple: IFoo {}
            impl IFoo {}
        },
        "Impl must be for an interface",
    );
}

#[test]
#[cfg(disabled)] // TODO: This should fail
fn err_method() {
    parse_class_err(
        quote! {
            pub class Simple: IFoo {}
            impl IFoo for Simple {
                fn zap() {}
            }
        },
        "xxx",
    );
}

#[test]
#[cfg(disabled)] // TODO: This should fail
fn err_method_ref_mut_self() {
    parse_class_err(
        quote! {
            pub class Simple: IFoo {}
            impl IFoo for Simple {
                fn zap(&mut self) {}
            }
        },
        "xxx",
    );
}

#[test]
fn err_unrecognized_attribute() {
    parse_class_err(
        quote! {
            #[bogus]
            pub class Simple: IFoo {}
            impl IFoo for Simple {
                fn zap(&mut self) {}
            }
        },
        "Unrecognized attribute",
    );
}

#[test]
fn derive_debug() {
    let class = parse_class_ok(quote! {
        #[derive(Debug)]
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert!(class.impl_debug);
}

#[test]
fn no_derive_debug() {
    let class = parse_class_ok(quote! {
        pub class Simple: IFoo {}
        impl IFoo for Simple {}
    });
    assert!(!class.impl_debug);
}

#[test]
fn err_derive_unrecognized() {
    parse_class_err(
        quote! {
            #[derive(Unknown)]
            pub class Simple: IFoo {}
            impl IFoo for Simple {}
        },
        "Unrecognized derive attribute",
    );
}

#[test]
fn from_impls() {
    let _class = parse_class_ok(quote! {
        class Server: IBar(IFoo), IZap(IFoo) {}
        impl IFoo for Server {}
        impl IBar for Server {}
        impl IZap for Server {}
    });
}
