use anyhow::Result;

pub fn enable_level_9_for_entry(&mut self, entry_id: u32) -> Result<()>;
pub fn disable_level_9_for_entry(&mut self, entry_id: u32);
pub fn is_entry_level_9(&self, entry_id: u32) -> bool;
