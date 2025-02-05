# ash-structure-utils

This repo includes helpful generated functions that help with working with structs in ash.
This is currently up to date for ash version 0.38.0+1.3.281.

# Current Usage
Currently, this repository only provides the names and sizes of vulkan structure types. Each of these functions has a feature, type_names and type_sizes, which are both enabled by default.

## Type names

To get the name of a struct from its `ash::vk::StructureType`, import the module `ash_structure_utils::type_names`. This allows you to cast to `VulkanNamed` which lets you call `.get_type_name()` on a structure type. This returns the enum name of the C structure type, as if you got it from the original VkStructureType enum. This is useful for creating debug statements.

## Type sizes

To get the size of a struct from its `ash::vk::StructureType`, import the module `ash-structure-utils::type_sizes`. This allows you to cast to `VulkanSized` which lets you call `.get_type_size()` on a structure type. This returns a usize equal to the amount of bytes fo the required structure type. This is useful for cloning unknown `ash::vk::BaseOutStructure`s or `ash::vk::BaseInStructure`s, which can be used for feature resolution.
