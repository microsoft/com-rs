use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::ItemStruct;

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
    quote!()
}
