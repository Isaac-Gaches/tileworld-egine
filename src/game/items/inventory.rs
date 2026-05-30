use crate::game::items::item_registry::ItemID;

pub struct ItemStack{
    pub count: u32,
    pub id: ItemID,
}

pub struct Inventory{
    slots: Vec<Option<ItemStack>>
}

impl Inventory{
    pub fn new()->Self{
        Self{
            slots: vec![],
        }
    }
}