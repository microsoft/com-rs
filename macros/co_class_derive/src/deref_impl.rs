use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemStruct,};

// impl std::ops::Deref for BritishShortHairCat {
//     type Target = InitBritishShortHairCat;
//     fn deref(&self) -> &Self::Target {
//         &self.__init_struct
//     }
// }
// impl std::ops::DerefMut for BritishShortHairCat {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.__init_struct
//     }
// }

pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_ident = macro_utils::get_real_ident(init_ident);
    let inner_init_field_ident = macro_utils::get_inner_init_field_ident();

    quote!(
        impl std::ops::Deref for #real_ident {
            type Target = #init_ident;
            fn deref(&self) -> &Self::Target {
                &self.#inner_init_field_ident
            }
        }

        impl std::ops::DerefMut for #real_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#inner_init_field_ident
            }
        }
    )
}