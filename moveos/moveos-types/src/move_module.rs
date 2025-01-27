// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state::MoveState;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

/// `MoveModule` is represented `moveos_std::move_module::MoveModule` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoveModule {
    pub byte_codes: Vec<u8>,
}

impl MoveModule {
    pub fn new(byte_codes: Vec<u8>) -> Self {
        Self { byte_codes }
    }
}

impl MoveStructType for MoveModule {
    const MODULE_NAME: &'static IdentStr = ident_str!("move_module");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MoveModule");
}

impl MoveState for MoveModule {
    fn move_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }
}
