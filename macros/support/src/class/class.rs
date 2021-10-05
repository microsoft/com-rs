use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use syn::parse::ParseBuffer;

#[derive(Debug)]
pub struct Class {
    pub name: Ident,
    pub has_class_factory: bool,
    pub docs: Vec<syn::Attribute>,
    pub visibility: syn::Visibility,
    pub interfaces: Vec<Interface>,
    pub methods: HashMap<syn::Path, Vec<InterfaceMethod>>,
    pub fields: Vec<syn::Field>,
    pub impl_debug: bool,
}

#[derive(Debug)]
pub struct InterfaceMethod {
    pub item: syn::ImplItemMethod,
    /// The original ident of the method definition. If the method has been
    /// renamed (to avoid collisions), then this will be the original ident as
    /// written by the user.
    pub original_ident: Ident,
}

impl Class {
    pub fn to_tokens(&self) -> TokenStream {
        let mut out: Vec<TokenStream> = Vec::new();
        out.push(self.to_struct_tokens());
        out.push(self.to_class_trait_impl_tokens());
        out.push(super::class_factory::generate(self));

        TokenStream::from_iter(out)
    }

    /// Get the paths of all interfaces including parent interfaces
    fn interfaces_paths<'a>(&'a self) -> HashSet<&'a syn::Path> {
        fn get_interface<'a>(interface: &'a Interface, result: &mut HashSet<&'a syn::Path>) {
            result.insert(&interface.path);
            if let Some(i) = &interface.parent {
                get_interface(i, result);
            }
        }
        let mut result = HashSet::new();
        for i in &self.interfaces {
            get_interface(i, &mut result)
        }
        result
    }

    /// Parse the class macro syntax (without the `impl`s)
    fn parse_class(
        input: syn::parse::ParseStream,
        docs: Vec<syn::Attribute>,
        has_class_factory: bool,
    ) -> syn::Result<Self> {
        let mut interfaces: Vec<Interface> = Vec::new();
        let visibility = input.parse::<syn::Visibility>()?;

        let _ = input.parse::<keywords::class>()?;
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<syn::Token!(:)>()?;

        while !input.peek(syn::token::Brace) {
            let path = input.parse::<syn::Path>()?;
            let interface = Interface {
                path: path.clone(),
                parent: None,
            };
            if interfaces.iter().any(|i| i.path == path) {
                return Err(syn::Error::new(path.span(), "interface was redefined"));
            }
            interfaces.push(interface);

            let mut current = interfaces.last_mut().unwrap();
            fn parse_parens(buffer: &ParseBuffer, current: &mut Interface) -> syn::Result<()> {
                while buffer.peek(syn::token::Paren) {
                    let contents;
                    syn::parenthesized!(contents in buffer);
                    let path = contents.parse::<syn::Path>()?;
                    let parent = Interface { path, parent: None };
                    current.parent = Some(Box::new(parent));
                    if !contents.is_empty() {
                        parse_parens(&contents, current.parent.as_mut().unwrap().as_mut())?;
                    }
                }

                Ok(())
            }

            parse_parens(&input, &mut current)?;

            if !input.peek(syn::token::Brace) {
                let _ = input.parse::<syn::Token!(,)>()?;
            }
        }
        let fields;
        syn::braced!(fields in input);
        let fields =
            syn::punctuated::Punctuated::<syn::Field, syn::Token!(,)>::parse_terminated_with(
                &fields,
                syn::Field::parse_named,
            )?;
        let fields = fields.into_iter().collect();

        Ok(Class {
            name,
            has_class_factory,
            docs,
            visibility,
            interfaces,
            methods: HashMap::new(),
            fields,
            impl_debug: false,
        })
    }

    /// The COM class object struct and `impl`
    ///
    /// Structure of the object:
    /// ```rust
    /// pub struct ClassName {
    ///     // ..interface vpointers..
    ///     // ..ref count..
    ///     // ..user defined fields..
    /// }
    /// ```
    pub fn to_struct_tokens(&self) -> TokenStream {
        let name = &self.name;
        let vis = &self.visibility;

        let interfaces = &self.interfaces;
        let interface_fields = interfaces.iter().enumerate().map(|(index, interface)| {
            let interface_name = &interface.path;
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                #field_ident: ::core::ptr::NonNull<<#interface_name as ::com::Interface>::VTable>
            }
        });
        let ref_count_ident = crate::utils::ref_count_ident();

        let user_fields = &self.fields;
        let docs = &self.docs;
        let methods = self.methods.values().flatten().map(|m| {
            let m_item = &m.item;
            quote! {
                #[allow(non_snake_case)]
                #m_item
            }
        });

        let iunknown = super::iunknown_impl::IUnknown::new();
        let add_ref = iunknown.to_add_ref_tokens();
        let query_interface = iunknown.to_query_interface_tokens(interfaces);
        let constructor = super::class_constructor::generate(self);
        let interface_drops = interfaces.iter().enumerate().map(|(index, _)| {
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                let _ = ::com::alloc::boxed::Box::from_raw(self.#field_ident.as_ptr());
            }
        });
        let debug = self.debug();
        let safe_query_interface = self.safe_query_interface();

        quote! {
            #(#docs)*
            #[repr(C)]
            #vis struct #name {
                #(#interface_fields,)*
                #ref_count_ident: ::core::cell::Cell<u32>,
                #(#user_fields),*
            }
            impl #name {
                #constructor
                #(#methods)*
                #add_ref
                #query_interface
                #safe_query_interface
            }
            #debug
            impl ::core::ops::Drop for #name {
                fn drop(&mut self) {
                    unsafe {
                        #(#interface_drops)*
                    }
                }
            }
        }
    }

    pub fn to_class_trait_impl_tokens(&self) -> TokenStream {
        let name = &self.name;
        let factory = if self.has_class_factory {
            let ident = crate::utils::class_factory_ident(name);
            quote! { #ident }
        } else {
            quote! { () }
        };
        let ref_count_ident = crate::utils::ref_count_ident();

        quote! {
            unsafe impl com::production::Class for #name {
                type Factory = #factory;

                fn dec_ref_count(&self) -> u32 {
                    let count = self.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
                    self.#ref_count_ident.set(count);
                    count
                }
            }
        }
    }

    fn debug(&self) -> TokenStream {
        if !self.impl_debug {
            return TokenStream::new();
        }

        let name = &self.name;
        let fields = self.fields.iter().map(|f| {
            let name = f.ident.as_ref().unwrap();
            quote! {
                .field(::core::stringify!(#name), &self.#name)
            }
        });

        quote! {
            impl ::core::fmt::Debug for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(::core::stringify!(#name))
                        #(#fields)*
                        .finish()
                }
            }
        }
    }

    fn safe_query_interface(&self) -> TokenStream {
        quote! {
            pub fn query_interface<T: ::com::Interface>(self: &::core::pin::Pin<::com::alloc::boxed::Box<Self>>) -> Option<T> {
                let mut result = None;
                let hr = unsafe { self.QueryInterface(&T::IID, &mut result as *mut _ as _) };

                if ::com::sys::FAILED(hr) {
                    return None;
                }
                debug_assert!(result.is_some(), "Successful call to query_interface yielded a null pointer");
                result
            }
        }
    }
}

impl syn::parse::Parse for Class {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class = None;
        let mut methods: HashMap<syn::Path, Vec<InterfaceMethod>> = HashMap::new();
        let mut impl_debug = false;
        while !input.is_empty() {
            let attributes = input.call(syn::Attribute::parse_outer)?;
            let mut docs = Vec::with_capacity(attributes.len());
            let mut has_class_factory = true;
            for attr in attributes {
                if attr.path.is_ident("doc") {
                    docs.push(attr)
                } else if attr.path.is_ident("no_class_factory") {
                    has_class_factory = false;
                } else if attr.path.is_ident("derive") {
                    parse_derive_debug(&attr)?;
                    impl_debug = true;
                } else {
                    return Err(syn::Error::new(attr.path.span(), "Unrecognized attribute"));
                }
            }

            if !input.peek(syn::Token!(impl)) {
                class = Some(Self::parse_class(input, docs, has_class_factory)?);
            } else {
                let item = input.parse::<syn::ItemImpl>()?;
                // TODO: ensure that class idents line up
                let interface = match item.trait_ {
                    Some((_, i, _)) => i,
                    None => {
                        return Err(syn::Error::new(
                            item.span(),
                            "Impl must be for an interface",
                        ))
                    }
                };
                let ms = item
                    .items
                    .into_iter()
                    .map(|i| match i {
                        syn::ImplItem::Method(m) => Ok(InterfaceMethod {
                            original_ident: m.sig.ident.clone(),
                            item: m,
                        }),
                        _ => Err(syn::Error::new(
                            i.span(),
                            "only trait methods are allowed when implementing an interface",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;

                if methods.insert(interface.clone(), ms).is_some() {
                    return Err(syn::Error::new(interface.span(), "interface was redefined"));
                }
            }
        }
        let mut class = match class {
            Some(c) => {
                let mut interface_paths = c.interfaces_paths();
                for i in methods.keys() {
                    if !interface_paths.remove(i) {
                        return Err(syn::Error::new(
                            i.span(),
                            "impl for a non-declared interface",
                        ));
                    }
                }
                if let Some(i) = interface_paths.into_iter().next() {
                    return Err(syn::Error::new(i.span(), "impl for interface is missing"));
                }
                c
            }
            None => {
                return Err(syn::Error::new(input.span(), "no class was defined"));
            }
        };
        class.impl_debug = impl_debug;
        class.methods = methods;
        find_method_name_collisions(&mut class);
        Ok(class)
    }
}

/// Resolve name collisions among methods defined on different interfaces, by
/// renaming some methods with a disambiguating suffix.
///
/// It is common for designers of COM interfaces to define multiple versions of
/// the same interface, where each new interface is a superset of the previous
/// version. For examples, see these DirectWrite interfaces:
///
/// * [IDWriteTextFormat](https://docs.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextformat)
/// * [IDWriteTextFormat1](https://docs.microsoft.com/en-us/windows/win32/api/dwrite_2/nn-dwrite_2-idwritetextformat1)
/// * [IDWriteTextFormat2](https://docs.microsoft.com/en-us/windows/win32/api/dwrite_3/nn-dwrite_3-idwritetextformat2)
/// * [IDWriteTextFormat3](https://docs.microsoft.com/en-us/windows/win32/api/dwrite_3/nn-dwrite_3-idwritetextformat3)
///
/// These interfaces define methods that have the same name. When implementing a
/// COM object in Rust, we need a way to distinguish these functions. The ugly
/// way is to require that different COM interface definitions avoid using the
/// same name, but that is clearly not feasible.
///
/// This implementation looks for name collisions, and simply renames methods
/// that participate in name collisions by appending the name of the interface.
/// If _that_ fails, then we use a numeric suffix. (We could just use the IID
/// as a suffix, but that's a harsh experience when debugging.)
fn find_method_name_collisions(class: &mut Class) {
    // First, we find the set of method names that collide. These are the method
    // names as defined on the COM interfaces. We just count how many times
    // each method name occurs; a count of more than 1 indicates a collision.
    // This pass uses `class.methods.values()` because we don't need to consider
    // the interface name.
    let mut method_name_count: HashMap<Ident, u32> = HashMap::new();
    for interface_methods in class.methods.values() {
        for m in interface_methods.iter() {
            *method_name_count
                .entry(m.original_ident.clone())
                .or_default() += 1;
        }
    }

    // Next, we scan the methods again. For each method defined on each
    // interface, we check whether this _bare_ method name was involved in a
    // collision. For example, if a COM class implements both `IFoo::zap()`
    // and `IBar::zap()`, then the name `zap` will have a collision.
    // Equivalently, `method_name_count["zap"] > 1`.
    //
    // If we find a method that has collided in this way, then we want to rename
    // the method _implementation_ (the function body provided by the definition
    // of the COM class), so that the renamed method does not collide with
    // anything. At the same time, we want the renamed method to be based on
    // the method name that was provided by the user, and also be based on the
    // interface name, so that callstacks shown in a debugger are sensible.
    //
    // To do so, we generate a new method name, using
    // `{old_method_name}__{interface_name}`. In our example above, the two
    // different `zap` methods will be named `IFoo__zap` and `IBar__zap`.
    // This gives a fairly good debugging experience, in case of a collision.
    // The method name resembles the `<MyClass as IFoo>::zap` form of a method
    // defined on a trait impl.
    //
    // There is one more case to consider, unfortunately. It is possible that
    // some COM interface defines a method with a name that collides with the
    // name that we just generated. In other words, there could be an interface
    // that defines a method named `IFoo__zap`, which would collide with one of
    // the names that we generated to avoid a collision in the first place.
    // (This situation is unlikely, but certainly possible. This could occur,
    // for example, if the COM interface definitions were themselves machine-
    // generated.)
    //
    // To handle that situation, we check whether our generated name (e.g.
    // `IFoo__zap`) collides with an existing name. If it does, then we append
    // a numeric suffix (using `collision_counter`). If that also collides, we
    // keep increasing the collision counter until we finally find one that
    // does not. We're good for up to 4 billion collisions, this way.

    let mut collision_counter: u32 = 0;

    for (interface, methods) in class.methods.iter_mut() {
        for method in methods.iter_mut() {
            // We know the unwrap() will succeed, because we're repeating the
            // same query that we just performed, above.
            let old_ident = &method.original_ident;
            let collides = *method_name_count.get(old_ident).unwrap() > 1;
            if !collides {
                // This is the normal case, where this method did not collide.
                // We don't have to do anything special in this case.
                continue;
            }

            // We've found a collision, such as `IFoo::zap` and `IBar::zap`.
            // (We'll enter this code for both method definitions.)
            // We try to fix the collision by renaming the method definitions,
            // by prepending the name of the interface itself. So we rename
            // the `zap` defined on `IFoo` to `IFoo__zap`.
            let interface_ident = path_to_single_string(interface);
            let new_ident_string = format!("{}__{}", interface_ident, old_ident);
            let mut new_ident = Ident::new(&new_ident_string, old_ident.span());

            // This checks for the pathological case described above, where
            // the generated `IFoo__zap` _also_ collides with some method.
            // This should never occur in practice, but we're prepared for it,
            // just in case.
            if method_name_count.contains_key(&new_ident) {
                loop {
                    assert!(collision_counter < std::u32::MAX);
                    new_ident = Ident::new(
                        &format!("{}__{:04}", new_ident_string, collision_counter),
                        old_ident.span(),
                    );
                    collision_counter += 1;
                    if !method_name_count.contains_key(&new_ident) {
                        break;
                    }
                }
            }

            // Modify the ident in the item definition (the function body),
            // because we're going to re-emit the entire function body definition.
            // It's easier to modify it here than to clone and edit it later.
            method.item.sig.ident = new_ident;
        }
    }
}

/// Converts a `Path` to a string, flattening each path segment and separating
/// them with `_`.
///
/// This function requires that each segment of the path have no generic
/// arguments.
fn path_to_single_string(path: &syn::Path) -> String {
    assert!(!path.segments.is_empty());
    let seg0 = &path.segments[0];
    assert!(seg0.arguments.is_empty());
    if path.segments.len() == 1 {
        seg0.ident.to_string()
    } else {
        let mut s = String::new();
        for (i, seg) in path.segments.iter().enumerate() {
            assert!(seg.arguments.is_empty());
            if i > 0 {
                s.push_str("_");
            }
            s.push_str(&seg.ident.to_string());
        }
        s
    }
}

fn parse_derive_debug(attr: &syn::Attribute) -> syn::Result<()> {
    match attr.parse_meta() {
        Ok(syn::Meta::List(l))
            if matches!(l.nested.iter().next(), Some(syn::NestedMeta::Meta(syn::Meta::Path(p))) if p.is_ident("Debug"))
                && l.nested.len() == 1 =>
        {
            Ok(())
        }
        _ => Err(syn::Error::new(
            attr.tokens.span(),
            "Unrecognized derive attribute",
        )),
    }
}

mod keywords {
    syn::custom_keyword!(class);
    syn::custom_keyword!(factory);
}

#[derive(Debug)]
pub struct Interface {
    pub path: syn::Path,
    pub parent: Option<Box<Interface>>,
}

impl Interface {
    /// Creates an intialized VTable for the interface
    pub fn to_initialized_vtable_tokens(&self, class: &Class, offset: usize) -> TokenStream {
        let class_name = &class.name;
        let vtable_ident = self.vtable_ident();
        let vtable_type = self.to_vtable_type_tokens();
        let parent = match self.parent.as_ref() {
            Some(p) => p.to_initialized_vtable_tokens(class, offset),
            None => Self::iunknown_tokens(class, offset),
        };
        let fields = class.methods.get(&self.path).unwrap().iter().map(|m| {
            let original_name = &m.original_ident;
            let name = &m.item.sig.ident;
            let params = m.item.sig.inputs.iter().filter_map(|p| {
                match p {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(p) => Some(p),
                }
            });
            let args = params.clone().map(|p| &p.pat);
            let translation = params.clone().map(|p| {
                let pat = &p.pat;
                let typ = &p.ty;
                quote! {
                    let #pat = <#typ as ::com::AbiTransferable>::from_abi(#pat);
                }
            });
            let params = params.map(|p| {
                let pat = &p.pat;
                let typ = &p.ty;
                quote! {
                    #pat: <#typ as ::com::AbiTransferable>::Abi
                }
            });
            let ret = &m.item.sig.output;
            let method = quote! {
                #[allow(non_snake_case)]
                unsafe extern "system" fn #name(this: ::core::ptr::NonNull<::core::ptr::NonNull<#vtable_ident>>, #(#params),*) #ret {
                    let this = this.as_ptr().sub(#offset);
                    let this = ::core::mem::ManuallyDrop::new(::com::production::ClassAllocation::from_raw(this as *mut _ as *mut #class_name));
                    #(#translation)*
                    #class_name::#name(&this, #(#args),*)
                }
            };
            let field_name = Ident::new(&crate::utils::snake_to_camel(&original_name.to_string()), proc_macro2::Span::call_site());
            quote! {
                #field_name: {
                    #method
                    #name
                }
            }
        });
        quote! {
            {
                #vtable_type
                #vtable_ident {
                    parent: #parent,
                    #(#fields),*
                }
            }
        }
    }

    fn to_vtable_type_tokens(&self) -> TokenStream {
        let name = &self.path;
        let vtable_ident = self.vtable_ident();
        quote! {
            type #vtable_ident = <#name as ::com::Interface>::VTable;
        }
    }

    fn vtable_ident(&self) -> proc_macro2::Ident {
        let name = &self.path;
        quote::format_ident!("{}VTable", name.segments.last().unwrap().ident)
    }

    fn iunknown_tokens(class: &Class, offset: usize) -> TokenStream {
        let iunknown = super::iunknown_impl::IUnknownAbi::new(class.name.clone(), offset);
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens();
        let query_interface = iunknown.to_query_interface_tokens();
        quote! {
            {
                type IUknownVTable = <::com::interfaces::IUnknown as ::com::Interface>::VTable;
                #add_ref
                #release
                #query_interface
                IUknownVTable {
                    AddRef,
                    Release,
                    QueryInterface,
                }
            }
        }
    }
}
