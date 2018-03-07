use logpack::decoder::Callbacks;
use logpack::decoder::TypeNameId;

use ansi_term::ANSIString;
use ansi_term::Colour::RGB;
use ansi_term::Colour;

pub struct Repr<'a> {
    output: &'a mut Vec<ANSIString<'static>>,
    enum_names: bool,
}

impl<'a> Repr<'a> {
    pub fn new(output: &'a mut Vec<ANSIString<'static>>) -> Self {
        let enum_names = false;
        Self { output, enum_names }
    }

    pub fn with_enum_names(self) -> Self {
        Self { enum_names: true, ..self }
    }
}

static NUM: Colour       = RGB(255, 200,   0);
static STR: Colour       = RGB(0,   192, 255);
static VOID: Colour      = RGB(80,   80,  80);
static PUNCT: Colour     = RGB(255, 255, 180);
static OPT: Colour       = RGB(100, 200, 100);
static VALNAME: Colour   = RGB(150, 250, 50);
static OPT_NEG: Colour   = RGB(200, 100, 100);
static FIELDNAME: Colour = RGB(180, 180, 255);
static TYPENAME: Colour  = RGB(150, 250, 0);

impl<'a> Callbacks for Repr<'a> {
    type SubType = Repr<'a>;

    fn handle_u8(&mut self, val: u8) {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_u16(&mut self, val: u16) {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_u32(&mut self, val: u32)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_u64(&mut self, val: u64)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_i8(&mut self, val: i32)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_i16(&mut self, val: i32)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_i32(&mut self, val: i32)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_i64(&mut self, val: i64)   {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_bool(&mut self, val: bool)  {
        self.output.push(NUM.paint(val.to_string()));
    }

    fn handle_string(&mut self, val: &str)  {
        self.output.push(STR.paint(format!("{:?}", val)));
    }

    fn handle_unit(&mut self)  {
        self.output.push(VOID.paint("()".to_string()));
    }

    fn handle_phantom(&mut self)  {
        self.output.push(VOID.paint("PhantomData".to_string()));
    }

    fn begin_enum(&mut self, typename_id: &TypeNameId, option_name: &String) -> &mut Self::SubType {
        if self.enum_names {
            self.output.push(TYPENAME.paint(typename_id.0.to_string()));
            self.output.push(PUNCT.bold().paint("::".to_string()));
        }
        self.output.push(OPT.paint(option_name.clone()));

        self
    }

    fn end_enum(&mut self, _typename_id: &TypeNameId) {
    }

    fn option_none(&mut self) {
        self.output.push(OPT_NEG.paint("None".to_string()));
    }

    fn option_some(&mut self) -> &mut Self::SubType  {
        self.output.push(OPT.paint("Some".to_string()));
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }

    fn option_end(&mut self) {
        self.output.push(PUNCT.paint(")".to_string()));
    }

    fn result_ok(&mut self) -> &mut Self::SubType {
        self.output.push(OPT.paint("Ok".to_string()));
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }

    fn result_err(&mut self) -> &mut Self::SubType {
        self.output.push(OPT_NEG.paint("Err".to_string()));
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }

    fn result_end(&mut self) {
        self.output.push(PUNCT.paint(")".to_string()));
    }

    fn struct_unit(&mut self, typename_id: Option<&TypeNameId>) {
        if let Some(typename_id) = typename_id {
            self.output.push(VALNAME.paint(typename_id.0.clone()));
        }
    }

    fn begin_struct_named(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType {
        if let Some(typename_id) = typename_id {
            self.output.push(VALNAME.paint(typename_id.0.clone()));
        }
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }

    fn begin_named_field(&mut self, field_idx: u16, field_name: &String) -> &mut Self::SubType {
        if field_idx != 0 {
            self.output.push(PUNCT.paint(", ".to_string()));
        }
        self.output.push(FIELDNAME.paint(field_name.clone()));
        self.output.push(PUNCT.paint(": ".to_string()));
        self
    }

    fn end_named_field(&mut self) {
    }

    fn end_struct_named(&mut self) {
        self.output.push(PUNCT.paint(")".to_string()));
    }

    fn begin_struct_tuple(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType {
        if let Some(typename_id) = typename_id {
            self.output.push(VALNAME.paint(typename_id.0.clone()));
        }
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }

    fn begin_tuple_field(&mut self, field_idx: u16) -> &mut Self::SubType {
        if field_idx != 0 {
            self.output.push(PUNCT.paint(", ".to_string()));
        }
        self
    }

    fn end_tuple_field(&mut self) {
    }

    fn end_struct_tuple(&mut self) {
        self.output.push(PUNCT.paint(")".to_string()));
    }

    fn begin_tuple(&mut self, _size: usize) -> &mut Self::SubType {
        self.output.push(PUNCT.paint("(".to_string()));
        self
    }
    fn begin_tuple_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            self.output.push(PUNCT.paint(", ".to_string()));
        }
    }

    fn end_tuple_item(&mut self) {
    }

    fn end_tuple(&mut self) {
        self.output.push(PUNCT.paint(")".to_string()));
    }

    fn begin_array(&mut self, _size: usize) -> &mut Self::SubType {
        self.output.push(PUNCT.paint("[".to_string()));
        self
    }
    fn begin_array_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            self.output.push(PUNCT.paint(", ".to_string()));
        }
    }
    fn end_array_item(&mut self) {
    }
    fn end_array(&mut self) {
        self.output.push(PUNCT.paint("]".to_string()));
    }

    fn begin_slice(&mut self, _size: usize) -> &mut Self::SubType {
        self.output.push(PUNCT.paint("[".to_string()));
        self
    }
    fn begin_slice_item(&mut self, field_idx: u16) {
        if field_idx != 0 {
            self.output.push(PUNCT.paint(", ".to_string()));
        }
    }
    fn end_slice_item(&mut self) {
    }
    fn end_slice(&mut self) {
        self.output.push(PUNCT.paint("]".to_string()));
    }
}
