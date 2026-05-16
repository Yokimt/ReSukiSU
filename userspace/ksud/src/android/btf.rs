use btf_rs::{Btf, BtfType, Type};
use std::path::Path;
pub struct BtfStruct {
    btf: Btf,
}

impl BtfStruct {
    pub fn init<P: AsRef<Path>>(path: P) -> Self {
        BtfStruct {
            btf: Btf::from_file(path.as_ref()).expect("btf初始化失败"),
        }
    }
    pub fn get_member_offset(&self, struct_name: &str, member_name: &str) -> Result<u32, String> {
        let types = self
            .btf
            .resolve_types_by_name(struct_name)
            .map_err(|e| format!("查找类型失败: {e}"))?;

        let struct_type = types
            .iter()
            .find_map(|t| {
                if let Type::Struct(s) = t {
                    Some(s)
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("未找到结构体: {struct_name}"))?;

        for member in &struct_type.members {
            let name = self
                .btf
                .resolve_name(member)
                .map_err(|e| format!("解析成员名称失败: {e}"))?;
            if name == member_name {
                let offset_in_bytes = (member.bit_offset() / 8) as u32;
                return Ok(offset_in_bytes);
            }
        }
        Err(format!("成员 '{member_name}' 未找到"))
    }
    //递归查找结构体内部
    pub fn get_member_offset_deep(
        &self,
        struct_name: &str,
        member_name: &str,
    ) -> Result<u32, String> {
        if let Ok(offset) = self.get_member_offset(struct_name, member_name) {
            return Ok(offset);
        }

        let types = self
            .btf
            .resolve_types_by_name(struct_name)
            .map_err(|e| format!("查找类型失败: {e}"))?;

        let struct_type = types
            .iter()
            .find_map(|t| {
                if let Type::Struct(s) = t {
                    Some(s)
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("未找到结构体: {struct_name}"))?;

        for member in &struct_type.members {
            let name = self.btf.resolve_name(member).unwrap_or_default();
            if name.is_empty() {
                if let Some(type_id) = member.get_type_id() {
                    if let Ok(Type::Struct(inner_struct)) = self.btf.resolve_type_by_id(type_id) {
                        for inner_member in &inner_struct.members {
                            let inner_name =
                                self.btf.resolve_name(inner_member).unwrap_or_default();
                            if inner_name == member_name {
                                // 注意：这里的偏移是相对于匿名结构体的，需要加上它在父结构体中的偏移
                                let offset_in_anon = inner_member.bit_offset() / 8;
                                let anon_base_offset = member.bit_offset() / 8;
                                return Ok((anon_base_offset + offset_in_anon) as u32);
                            }
                        }
                    }
                }
            }
        }

        Err(format!("成员 '{}' 未找到（含嵌套）", member_name))
    }
}
