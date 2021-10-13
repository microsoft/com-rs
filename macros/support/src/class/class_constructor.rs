use super::Class;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates a function used to instantiate the COM class
pub fn generate(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let parameters = &class.fields;
    let user_fields = class.fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name
        }
    });

    let ref_count_ident = crate::utils::ref_count_ident();

    // Generate the vptr field idents needed in the instantiation syntax of the COM struct.
    let interface_fields = class
        .interfaces
        .iter()
        .enumerate()
        .map(|(index, interface)| {
            let interface_field_ident = interface.chain_ident(index);
            let vtable_static_item = interface.vtable_static_item_ident(class);
            quote! {
                #interface_field_ident: &#vtable_static_item,
            }
        });

    quote! {
        /// Allocate the class casting it to the supplied interface
        ///
        /// This allocates the class on the heap and pins it. This is because COM classes
        /// must have a stable location in memory. Once a COM class is instantiated somewhere
        /// it must stay there.
        #vis fn allocate(#(#parameters),*) -> ::com::production::ClassAllocation<Self> {
            let instance = #name {
                #(#interface_fields)*
                #ref_count_ident: ::core::sync::atomic::AtomicU32::new(1),
                #(#user_fields),*
            };
            let instance = ::com::alloc::boxed::Box::pin(instance);
            ::com::production::ClassAllocation::new(instance)
        }
    }
}
