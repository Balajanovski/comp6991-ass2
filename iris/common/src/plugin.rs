use abi_stable::{
    declare_root_module_statics,
    library::{LibraryError, RootModule},
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RBox},
    StableAbi,
};

use crate::types::{PluginReply, PluginMsg, PluginName};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "PluginMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct PluginMod {
    pub new: extern "C" fn() -> PluginStateBox,
    pub pl_name: extern "C" fn() -> PluginName,
    pub handler: extern "C" fn(&mut PluginStateBox, PluginMsg) -> Option<PluginReply>,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginState;

pub type PluginStateBox = DynTrait<'static, RBox<()>, PluginState>;

impl RootModule for PluginMod_Ref {
    declare_root_module_statics! {PluginMod_Ref}
    const BASE_NAME: &'static str = "iris";
    const NAME: &'static str = "iris";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    fn initialization(self) -> Result<Self, LibraryError> {
        Ok(self)
    }
}