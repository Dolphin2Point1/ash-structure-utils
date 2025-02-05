// Heavily based on ash's generator. <https://github.com/ash-rs/ash/blob/master/generator/src/lib.rs>
// This means that this is licensed under MIT

use quote::quote;
use std::{collections::HashSet, path::Path};

use proc_macro2::{Ident, Span, TokenStream};
use vk_parse::{
    ExtensionChild, FeatureChild, InterfaceItem, RegistryChild, Type, TypeMember, TypeSpec,
    TypesChild,
};

macro_rules! get_variant {
    ($variant:path) => {
        |enum_| match enum_ {
            $variant(inner) => Some(inner),
            _ => None,
        }
    };
    ($variant:path { $($member:ident),+ }) => {
        |enum_| match enum_ {
            $variant { $($member),+, .. } => Some(( $($member),+ )),
            _ => None,
        }
    };
}

pub fn vkspec_name_to_ash_struct_name(struct_name: &str) -> &str {
    struct_name.strip_prefix("Vk").unwrap_or(struct_name)
}

pub fn generate_struct_type_size(t: &Type, blacklist: &HashSet<String>) -> Option<TokenStream> {
    if t.name.is_none() {
        return None;
    }
    let original_name = &t.name.as_ref()?[..];
    if blacklist.contains(original_name) {
        return None;
    }

    let name = vkspec_name_to_ash_struct_name(original_name);

    let members;
    if let TypeSpec::Members(tempmembers) = &t.spec {
        members = tempmembers;
    } else {
        return None;
    }
    let s_type_field = members
        .iter()
        .filter_map(get_variant!(TypeMember::Definition))
        .find(|def| def.code.starts_with("VkStructureType"))?;

    s_type_field.values.as_ref()?;

    let name_ident = Ident::new(name, Span::call_site());

    Some(quote! {
       ash::vk::#name_ident::STRUCTURE_TYPE => std::mem::size_of::<ash::vk::#name_ident>()
    })
}

pub fn generate_struct_name(t: &Type, blacklist: &HashSet<String>) -> Option<TokenStream> {
    if t.name.is_none() {
        return None;
    }
    let original_name = &t.name.as_ref()?[..];
    if blacklist.contains(original_name) {
        return None;
    }

    let name = vkspec_name_to_ash_struct_name(original_name);

    let members;
    if let TypeSpec::Members(tempmembers) = &t.spec {
        members = tempmembers;
    } else {
        return None;
    }
    let s_type_field = members
        .iter()
        .filter_map(get_variant!(TypeMember::Definition))
        .find(|def| def.code.starts_with("VkStructureType"))?;

    let s_type_name = s_type_field.values.as_ref()?;

    let name_ident = Ident::new(name, Span::call_site());

    Some(quote! {
       ash::vk::#name_ident::STRUCTURE_TYPE => #s_type_name
    })
}

pub fn write_source_code<P: AsRef<Path>>(vk_xml_path: &Path, src_dir: P) {
    use std::{fs::File, io::Write};

    let (spec, errors) = vk_parse::parse_file(vk_xml_path).expect("invalid vk.xml");
    let src_dir = src_dir.as_ref();

    if !errors.is_empty() {
        eprintln!("Parsed with errors: {:?}", errors);
    }

    let types: Vec<Type> = spec
        .0
        .iter()
        .filter_map(get_variant!(RegistryChild::Types))
        .flat_map(|x| x.children.clone())
        .filter_map(get_variant!(TypesChild::Type))
        .collect();

    let mut type_generation_blacklist = HashSet::new();
    let vulkansc = "vulkansc";
    // Vulkan SC is not supported by ash, so we must blacklist anything in it.
    type_generation_blacklist.extend(
        spec.0
            .iter()
            .filter_map(get_variant!(RegistryChild::Feature))
            .filter(|f| f.api == vulkansc)
            .flat_map(|f| &f.children)
            .filter_map(get_variant!(FeatureChild::Require { items }))
            .flat_map(|f| f)
            .filter_map(get_variant!(InterfaceItem::Type { name }))
            .map(|f| f.clone()),
    );
    type_generation_blacklist.extend(
        spec.0
            .iter()
            .filter_map(get_variant!(RegistryChild::Extensions))
            .flat_map(|e| &e.children)
            .flat_map(|e| &e.children)
            .filter_map(get_variant!(ExtensionChild::Require { api, items }))
            .filter(|(a, _b)| a.is_some() && a.as_ref().unwrap() == vulkansc)
            .flat_map(|(_a, b)| b)
            .filter_map(get_variant!(InterfaceItem::Type { name }))
            .map(|e| e.clone()),
    );
    type_generation_blacklist.extend(
        spec.0
            .iter()
            .filter_map(get_variant!(RegistryChild::Extensions))
            .flat_map(|e| &e.children)
            .filter(|e| e.supported.is_some() && e.supported.as_ref().unwrap() == vulkansc)
            .flat_map(|e| &e.children)
            .filter_map(get_variant!(ExtensionChild::Require { items }))
            .flatten()
            .filter_map(get_variant!(InterfaceItem::Type { name }))
            .map(|e| e.clone()),
    );

    let mut type_size_tokens = Vec::new();
    for t in &types {
        let option_type_size = generate_struct_type_size(&t, &type_generation_blacklist);
        if option_type_size.is_none() {
            continue;
        }
        type_size_tokens.push(option_type_size.unwrap());
    }
    let mut type_name_tokens = Vec::new();
    for t in &types {
        let option_type_name = generate_struct_name(&t, &type_generation_blacklist);
        if option_type_name.is_none() {
            continue;
        }
        type_name_tokens.push(option_type_name.unwrap());
    }

    let type_size_code = quote! {
        pub trait VulkanSized {
            fn get_type_size(self) -> usize;
        }

        impl VulkanSized for ash::vk::StructureType {
            fn get_type_size(self) -> usize {
                use ash::vk::TaggedStructure;
                match self {
                    #(#type_size_tokens),*,
                    _ => panic!("Error while trying to find type size of structure type numbered {}. No such structure type is known. You may have to update ash-structure-utils.", self.as_raw()),
                }
            }
        }
    };
    let type_name_code = quote! {
        pub trait VulkanNamed {
            fn get_type_name(self) -> &'static str;
        }

        impl VulkanNamed for ash::vk::StructureType {
            fn get_type_name(self) -> &'static str {
                use ash::vk::TaggedStructure;
                match self {
                    #(#type_name_tokens),*,
                    _ => panic!("Error while trying to find name of structure type numbered {}. No such structure type is known. You may have to update ash-structure-utils.", self.as_raw()),
                }
            }
        }
    };

    fn write_formatted(text: &[u8], out: File) -> std::process::Child {
        let mut child = std::process::Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(out)
            .spawn()
            .expect("Failed to spawn `rustfmt`");
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(text).unwrap();
        drop(stdin);
        child
    }

    let type_size_file =
        File::create(src_dir.join("type_sizes.rs")).expect("type_sizes.rs cannot be created.");
    let type_name_file =
        File::create(src_dir.join("type_names.rs")).expect("type_names.rs cannot be created.");

    let processes = [
        write_formatted(type_size_code.to_string().as_bytes(), type_size_file),
        write_formatted(type_name_code.to_string().as_bytes(), type_name_file),
    ];

    for mut p in processes {
        let status = p.wait().unwrap();
        assert!(status.success());
    }
}
