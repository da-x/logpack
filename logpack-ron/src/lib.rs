extern crate logpack;
extern crate ansi_term;

use logpack::decoder::Callbacks;
use logpack::decoder::TypeNameId;

pub mod ansi;

pub struct Repr<'a> {
    output: &'a mut String,
    enum_names: bool,
}

impl<'a> Repr<'a> {
    pub fn new(output: &'a mut String) -> Self {
        let enum_names = false;
        Self { output, enum_names }
    }

    pub fn with_enum_names(self) -> Self {
        Self { enum_names: true, ..self }
    }
}

impl<'a> Callbacks for Repr<'a> {
    type SubType = Repr<'a>;

    fn handle_u8(&mut self, val: u8) {
        *self.output += &val.to_string();
    }

    fn handle_u16(&mut self, val: u16) {
        *self.output += &val.to_string();
    }

    fn handle_u32(&mut self, val: u32)  {
        *self.output += &val.to_string();
    }

    fn handle_u64(&mut self, val: u64)  {
        *self.output += &val.to_string();
    }

    fn handle_i8(&mut self, val: i32)  {
        *self.output += &val.to_string();
    }

    fn handle_i16(&mut self, val: i32)  {
        *self.output += &val.to_string();
    }

    fn handle_i32(&mut self, val: i32)  {
        *self.output += &val.to_string();
    }

    fn handle_i64(&mut self, val: i64)   {
        *self.output += &val.to_string();
    }

    fn handle_bool(&mut self, val: bool)  {
        *self.output += &val.to_string();
    }

    fn handle_string(&mut self, val: &str)  {
        *self.output += &format!("{:?}", val);
    }

    fn handle_unit(&mut self)  {
        *self.output += &"()";
    }

    fn handle_phantom(&mut self)  {
        *self.output += &"PhantomData";
    }

    fn begin_enum(&mut self, typename_id: &TypeNameId, option_name: &String) -> &mut Self::SubType {
        if self.enum_names {
            *self.output += typename_id.0.as_str();
            *self.output += "::";
        }
        *self.output += option_name;

        self
    }

    fn end_enum(&mut self, _typename_id: &TypeNameId) {
    }

    fn option_none(&mut self) {
        *self.output += "None";
    }

    fn option_some(&mut self) -> &mut Self::SubType  {
        *self.output += "Some(";
        self
    }

    fn option_end(&mut self) {
        *self.output += ")";
    }

    fn result_ok(&mut self) -> &mut Self::SubType {
        *self.output += "Ok(";
        self
    }

    fn result_err(&mut self) -> &mut Self::SubType {
        *self.output += "Err(";
        self
    }

    fn result_end(&mut self) {
        *self.output += ")";
    }

    fn struct_unit(&mut self, typename_id: Option<&TypeNameId>) {
        if let Some(typename_id) = typename_id {
            *self.output += typename_id.0.as_str();
        }
    }

    fn begin_struct_named(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType {
        if let Some(typename_id) = typename_id {
            *self.output += typename_id.0.as_str();
        }
        *self.output += "(";
        self
    }
    fn begin_named_field(&mut self, field_idx: u16, field_name: &String) -> &mut Self::SubType {
        if field_idx != 0 {
            *self.output += ", ";
        }
        *self.output += field_name.as_str();
        *self.output += ": ";
        self
    }
    fn end_named_field(&mut self) {
    }
    fn end_struct_named(&mut self) {
        *self.output += ")";
    }

    fn begin_struct_tuple(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType {
        if let Some(typename_id) = typename_id {
            *self.output += typename_id.0.as_str();
        }
        *self.output += "(";
        self
    }
    fn begin_tuple_field(&mut self, field_idx: u16) -> &mut Self::SubType {
        if field_idx != 0 {
            *self.output += ", ";
        }
        self
    }
    fn end_tuple_field(&mut self) {
    }
    fn end_struct_tuple(&mut self) {
        *self.output += ")";
    }

    fn begin_tuple(&mut self, _size: usize) -> &mut Self::SubType {
        *self.output += "(";
        self
    }
    fn begin_tuple_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            *self.output += ", ";
        }
    }
    fn end_tuple_item(&mut self) {
    }
    fn end_tuple(&mut self) {
        *self.output += ")";
    }

    fn begin_array(&mut self, _size: usize) -> &mut Self::SubType {
        *self.output += "[";
        self
    }
    fn begin_array_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            *self.output += ", ";
        }
    }
    fn end_array_item(&mut self) {
    }
    fn end_array(&mut self) {
        *self.output += "]";
    }

    fn begin_slice(&mut self, _size: usize) -> &mut Self::SubType {
        *self.output += "[";
        self
    }
    fn begin_slice_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            *self.output += ", ";
        }
    }
    fn end_slice_item(&mut self) {
    }
    fn end_slice(&mut self) {
        *self.output += "]";
    }
}
