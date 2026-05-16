use crate::android::btf::BtfStruct;
use object::{ObjectSection, SymbolSection};
use object::{Endianness,Object, ObjectSymbol, read::elf::ElfFile64};
use regex_lite::Regex;
use std::fs;
pub struct ModulePatcher {
    btf: BtfStruct,
    data: Vec<u8>,
}

impl ModulePatcher {
    pub fn new(btf_path: &str, elf_path: &str) -> Self {
        Self {
            btf: BtfStruct::init(btf_path),
            data: fs::read(elf_path).unwrap(),
        }
    }
    pub fn patch_elf(&self, outpath: &str) -> Result<(), String>  {
        let mut patched_data = self.data.clone();
        let elf = ElfFile64::<Endianness>::parse(&*self.data).map_err(|e| format!("ELF 解析失败: {}", e))?;
        let re = Regex::new(r"^(.+?)__((?:[^_]+(?:_[^_]+)*?)?)__offset$").map_err(|e| format!("正则编译失败: {}", e))?;
        let mut requirements: Vec<(String, String, String)> = Vec::new();
        for sym in elf.symbols() {
            let name = sym.name().unwrap_or("");
            if let Some(caps) = re.captures(name) {
                let struct_name = caps[1].to_string();
                let member_name = caps[2].to_string();
                if !struct_name.is_empty() && !member_name.is_empty() {
                    requirements.push((name.to_string(), struct_name, member_name));
                }
            }
        }

        if requirements.is_empty() {
            return Err("模块中未找到任何符合命名规则的偏移符号".to_string());
        }
        for (symbol_name, struct_name, member_name) in &requirements {
            // 4.1 查询 BTF 获取偏移
            let offset = self
                .btf
                .get_member_offset_deep(struct_name, member_name)
                .map_err(|e| format!("获取 {struct_name}__{member_name} 偏移失败: {e}"))?;

            // 4.2 定位符号在文件中的位置
            let sym = elf
                .symbols()
                .find(|s| s.name() == Ok(symbol_name.as_str()))
                .ok_or_else(|| format!("符号 '{}' 未找到", symbol_name))?;

            let section_index = match sym.section() {
                SymbolSection::Section(idx) => idx,
                _ => return Err(format!("符号 '{}' 没有关联的节区", symbol_name)),
            };

            // 4.4 获取节对象（需要导入 ObjectSection trait）
            let section = elf
                .section_by_index(section_index)
                .map_err(|e| format!("无法通过索引获取节: {}", e))?;



            let section_virt_base = section.address();
            let (file_base, _file_size) = section
                .file_range()
                .ok_or(format!("节区没有文件范围"))?;
            let file_offset = (file_base as usize) + (sym.address() - section_virt_base) as usize;


            if file_offset + 8 > patched_data.len() {
                return Err(format!("符号 '{}' 偏移越界", symbol_name));
            }

            // 4.3 写入小端序 u64
            patched_data[file_offset..file_offset + 8]
                .copy_from_slice(&(offset as u64).to_le_bytes());
        }

        // 5. 写回文件
        fs::write(outpath, &patched_data).map_err(|e| format!("写入文件失败: {}", e))?;

        println!(
            "成功修补 {} 个偏移符号，输出: {}",
            requirements.len(),
            outpath
        );
        Ok(())
    }
}
