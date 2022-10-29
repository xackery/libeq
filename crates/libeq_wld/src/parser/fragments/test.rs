use super::StringReference;
use libeq_wld_macro::FragParser;

#[derive(FragParser)]
#[fragment_id = 0x01]
pub struct TestFragment {
    pub name_reference: StringReference,
    pub flags: u32,
    pub some_count: u32,
    #[count(some_count)]
    pub some_stuff: Vec<u32>,
    #[present_when(flags == 0x01)]
    pub something_else: f32,
    #[count(some_count)]
    #[present_when(flags == 0x01)]
    pub optional_thing: Option<Vec<u32>>,
}
